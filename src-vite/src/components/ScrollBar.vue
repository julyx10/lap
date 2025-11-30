<template>
  <div 
    :class="[
      'h-full flex flex-col items-center justify-center',
      markers.length > 0 ? 'w-16' : 'm-1 w-2'
    ]"
  >
    <TButton :icon="IconArrowUp" :buttonSize="'small'" @click="clickPreviousPage" class="shrink-0"/>
    
    <!-- custom area -->
    <div ref="trackRef" 
      :class="[
        'flex-1 w-full relative rounded-full', 
        total > pageSize ? 'bg-base-200 cursor-pointer' : 'bg-base-200/30'
      ]" 
      @mousedown="handleTrackClick"
      >
      <!-- Track background (optional, can be styled via class) -->
      
      <!-- Thumb -->
      <div v-if="total > pageSize"
        ref="thumbRef"
        class="absolute left-0 right-0 rounded-full bg-base-content/30 hover:bg-base-content/50 active:bg-base-content/70 transition-colors duration-200"
        :style="{ top: thumbTop + 'px', height: thumbHeight + 'px' }"
        @mousedown.stop="handleThumbMouseDown"
      ></div>

      <!-- Markers (Years and Months) -->
      <div v-for="(marker, index) in displayMarkers" :key="index"
        class="absolute right-0 pointer-events-none px-1 select-none flex items-center justify-end w-full"
        :class="[
          marker.isYear ? 'text-[10px] text-base-content/30' : 'text-[9px] text-base-content/30'
        ]"
        :style="{ top: marker.top + '%' }"
      >
        {{ marker.label }}
        <!-- Tick mark -->
        <div class="bg-base-content/30 ml-1" 
          :class="[
            marker.isYear ? 'h-[1px] w-[4px]' : (marker.isTick ? 'h-[1px] w-[1px]' : 'h-[1px] w-[4px]')
          ]"
        ></div>
      </div>
    </div>

    <TButton :icon="IconArrowDown" :buttonSize="'small'" @click="clickNextPage" class="shrink-0"/>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue';
import TButton from './TButton.vue';
import { IconArrowUp, IconArrowDown } from '@/common/icons';

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
  }
});

const emit = defineEmits(['update:modelValue', 'change']);

const trackRef = ref<HTMLElement | null>(null);
const thumbRef = ref<HTMLElement | null>(null);

// State for dragging
const isDragging = ref(false);
const startY = ref(0);
const startThumbTop = ref(0);

// Computed properties for thumb dimensions and position
const trackHeight = ref(0);

// Process markers to show Year/Month changes and Date ticks
const displayMarkers = computed(() => {
  if (!props.total || props.total === 0 || !props.markers.length) return [];
  
  const height = trackHeight.value; 
  const minLabelDistance = 12;
  const minTickDistance = 5;

  // 1. Generate Candidates
  // We identify the highest priority change for each position
  const candidates: Array<{ 
    label: string, 
    top: number, 
    type: 'year' | 'month' | 'date', 
    pixelPos: number,
    accepted: boolean 
  }> = [];

  let lastYear: number | null = null;
  let lastMonth: number | null = null;

  for (const marker of props.markers) {
    if (marker.position === undefined || marker.position === null) continue;
    
    const top = Math.min(100, (marker.position / props.total) * 100);
    const currentPixelPos = (top / 100) * height;
    
    const year = marker.year;
    const month = marker.month;

    if (year !== null && year !== lastYear) {
      // Year change
      candidates.push({
        label: year === 0 ? '' : String(year),
        top: top,
        type: 'year',
        pixelPos: currentPixelPos,
        accepted: false
      });
      lastYear = year;
      lastMonth = month;
    } else if (month !== null && month !== lastMonth) {
      // Month change
      candidates.push({
        label: String(month).padStart(2, '0'),
        top: top,
        type: 'month',
        pixelPos: currentPixelPos,
        accepted: false
      });
      lastMonth = month;
    } else {
      // Date change
      candidates.push({
        label: '',
        top: top,
        type: 'date',
        pixelPos: currentPixelPos,
        accepted: false
      });
    }
  }

  // 2. Filter Years
  // Check distance to previous accepted Year
  let lastAcceptedYearPos = -minLabelDistance * 2; // Ensure first one passes
  for (const item of candidates) {
    if (item.type === 'year') {
      if (Math.abs(item.pixelPos - lastAcceptedYearPos) >= minLabelDistance) {
        item.accepted = true;
        lastAcceptedYearPos = item.pixelPos;
      }
    }
  }

  // 3. Filter Months
  // Check distance to prev accepted (Year/Month) AND next accepted Year
  let lastAcceptedLabelPos = -minLabelDistance * 2;
  // We need to update lastAcceptedLabelPos as we go, but we also need to look ahead for Years.
  // Since Years are already decided, we can look ahead.
  
  // First, let's find the indices of accepted years for fast look-ahead
  const acceptedYearIndices = candidates
    .map((item, index) => (item.type === 'year' && item.accepted) ? index : -1)
    .filter(index => index !== -1);

  for (let i = 0; i < candidates.length; i++) {
    const item = candidates[i];
    
    if (item.type === 'year' && item.accepted) {
      lastAcceptedLabelPos = item.pixelPos;
      continue;
    }

    if (item.type === 'month') {
      // Check prev
      const distPrev = Math.abs(item.pixelPos - lastAcceptedLabelPos);
      
      // Check next Year
      // Find the first accepted year index that is > i
      const nextYearIndex = acceptedYearIndices.find(idx => idx > i);
      let distNext = Infinity;
      if (nextYearIndex !== undefined) {
        distNext = Math.abs(candidates[nextYearIndex].pixelPos - item.pixelPos);
      }

      if (distPrev >= minLabelDistance && distNext >= minLabelDistance) {
        item.accepted = true;
        lastAcceptedLabelPos = item.pixelPos;
      }
    }
  }

  // 4. Filter Dates
  // Check distance to prev accepted (Year/Month/Date) AND next accepted Label (Year/Month)
  let lastAcceptedItemPos = -minTickDistance * 2;
  
  // Find indices of all accepted labels (Years and Months) for look-ahead
  const acceptedLabelIndices = candidates
    .map((item, index) => ((item.type === 'year' || item.type === 'month') && item.accepted) ? index : -1)
    .filter(index => index !== -1);

  for (let i = 0; i < candidates.length; i++) {
    const item = candidates[i];
    
    if (item.accepted) {
      lastAcceptedItemPos = item.pixelPos;
      continue;
    }

    if (item.type === 'date') {
      // Check prev
      const distPrev = Math.abs(item.pixelPos - lastAcceptedItemPos);
      
      // Check next Label
      const nextLabelIndex = acceptedLabelIndices.find(idx => idx > i);
      let distNext = Infinity;
      if (nextLabelIndex !== undefined) {
        distNext = Math.abs(candidates[nextLabelIndex].pixelPos - item.pixelPos);
      }

      if (distPrev >= minTickDistance && distNext >= minTickDistance) {
        item.accepted = true;
        lastAcceptedItemPos = item.pixelPos;
      }
    }
  }

  // Return formatted result
  return candidates
    .filter(item => item.accepted)
    .map(item => ({
      label: item.label,
      top: item.top,
      isYear: item.type === 'year',
      isTick: item.type === 'date',
      pixelPos: item.pixelPos
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

function emitUpdate(newValue: number) {
  if (newValue !== props.modelValue) {
    emit('update:modelValue', newValue);
    emit('change', newValue);
  }
}

</script>

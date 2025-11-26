<template>
  <div class="w-full h-full flex flex-col items-center justify-center">
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
        class="absolute left-0 right-0 rounded-full bg-base-content/30 hover:bg-base-content/50 active:bg-base-content/70 transition-colors duration-150"
        :style="{ top: thumbTop + 'px', height: thumbHeight + 'px' }"
        @mousedown.stop="handleThumbMouseDown"
      ></div>

      <!-- Markers (Years) -->
      <div v-for="(marker, index) in markers" :key="index"
        class="absolute right-0 text-[10px] text-base-content/50 pointer-events-none pr-1 select-none"
        :style="{ top: (marker.position * 100) + '%' }"
      >
        {{ marker.label }}
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
    type: Array as () => Array<{ label: string, position: number }>, // position is 0.0 to 1.0
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

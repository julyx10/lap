<template>
  <Teleport to="body">
    <div class="fixed inset-0 bg-black/30 z-[600]">
      <div 
        ref="modalDialogRef"
        class="text-base-content/70 bg-base-200/80 backdrop-blur-md border border-base-content/30 rounded-box overflow-hidden"
        :style="{ position: 'fixed', top: y + 'px', left: x + 'px', width: width + 'px', ...(height !== undefined && { height: height + 'px' }) }"
      >
        <!-- title bar -->
        <div ref="titleBarRef" class="p-3 flex items-center justify-between select-none cursor-default">
          {{ title }}
          <TButton
            :icon="IconClose"
            :buttonSize="'small'"
            @click="clickCancel"
          />
        </div>

        <!-- dialog content -->
        <div class="p-3">
          <slot></slot>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { IconClose } from '@/common/icons';
import TButton from '@/components/TButton.vue';

const props = defineProps({
  title: { 
    type: String, 
    required: true
  },
  width: {
    type: Number,
    default: 400
  },
  height: {
    type: Number,
    default: undefined // undefined means no height limit
  }
});

const emit = defineEmits(['cancel']);

const modalDialogRef = ref<HTMLDivElement | null>(null);
const titleBarRef = ref<HTMLDivElement | null>(null);

const x = ref(0);
const y = ref(0);
const isDragging = ref(false);

let startX = 0;
let startY = 0;
let initialX = 0;
let initialY = 0;

const dragStart = (event: MouseEvent) => {
  isDragging.value = true;
  startX = event.clientX;
  startY = event.clientY;
  initialX = x.value;
  initialY = y.value;
  document.addEventListener('mousemove', dragMove);
  document.addEventListener('mouseup', dragEnd);
};

const dragMove = (event: MouseEvent) => {
  if (isDragging.value) {
    event.preventDefault();
    
    const dx = event.clientX - startX;
    const dy = event.clientY - startY;
    
    let newX = initialX + dx;
    let newY = initialY + dy;

    if (modalDialogRef.value) {
      const boxWidth = modalDialogRef.value.offsetWidth;
      const boxHeight = modalDialogRef.value.offsetHeight;
      const windowWidth = window.innerWidth;
      const windowHeight = window.innerHeight;

      if (newX < 0) newX = 0;
      if (newX > windowWidth - boxWidth) newX = windowWidth - boxWidth;
      if (newY < 0) newY = 0;
      if (newY > windowHeight - boxHeight) newY = windowHeight - boxHeight;
    }
    
    x.value = newX;
    y.value = newY;
  }
};

const dragEnd = () => {
  isDragging.value = false;
  document.removeEventListener('mousemove', dragMove);
  document.removeEventListener('mouseup', dragEnd);
};

const centerDialog = () => {
  if (modalDialogRef.value) {
    const boxWidth = modalDialogRef.value.offsetWidth;
    const boxHeight = modalDialogRef.value.offsetHeight;
    x.value = (window.innerWidth - boxWidth) / 2;
    y.value = (window.innerHeight - boxHeight) / 2;
  }
};

onMounted(() => {
  window.addEventListener('resize', clampPosition);
  nextTick(() => {
    centerDialog(); // Center on mount
  });
  if (titleBarRef.value) {
      titleBarRef.value.addEventListener('mousedown', dragStart as EventListener);
  }
});

onUnmounted(() => {
  window.removeEventListener('resize', clampPosition);

  if (titleBarRef.value) {
      titleBarRef.value.removeEventListener('mousedown', dragStart as EventListener);
  }
  // In case component is unmounted while dragging
  if (isDragging.value) {
    dragEnd();
  }
});

const clampPosition = () => {
  if (modalDialogRef.value) {
    const boxWidth = modalDialogRef.value.offsetWidth;
    const boxHeight = modalDialogRef.value.offsetHeight;
    const windowWidth = window.innerWidth;
    const windowHeight = window.innerHeight;

    let newX = x.value;
    let newY = y.value;

    if (newX < 0) newX = 0;
    if (newX > windowWidth - boxWidth) newX = windowWidth - boxWidth;
    if (newY < 0) newY = 0;
    if (newY > windowHeight - boxHeight) newY = windowHeight - boxHeight;
    
    x.value = newX;
    y.value = newY;
  }
};

const clickCancel = () => {
  emit('cancel');
};
</script>

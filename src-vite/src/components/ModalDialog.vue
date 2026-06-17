<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="visible" class="fixed inset-0 bg-black/30 z-600">
        <div
          ref="modalDialogRef"
          class="modal-dialog text-base-content/70 bg-base-200/80 backdrop-blur-md border border-base-content/30 rounded-box overflow-hidden flex flex-col"
          :style="{ position: 'fixed', top: y + 'px', left: x + 'px', width: width + 'px', ...(height !== undefined && { height: height + 'px' }) }"
        >
        <!-- title bar -->
        <div
          class="p-3 flex items-center justify-between select-none cursor-default shrink-0"
          @mousedown="dragStart"
        >
          {{ title }}
          <TButton
            :icon="IconClose"
            :buttonSize="'small'"
            @mousedown.stop
            @click="clickCancel"
          />
        </div>

        <!-- dialog content -->
        <div class="px-3 pb-3 flex-1 min-h-0 flex flex-col">
          <slot></slot>
        </div>
      </div>
    </div>
    </Transition>
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
const visible = ref(false);
onMounted(() => { visible.value = true; });

const x = ref(0);
const y = ref(0);
const isDragging = ref(false);
const visibleEdge = 96;

let startX = 0;
let startY = 0;
let initialX = 0;
let initialY = 0;

const dragStart = (event: MouseEvent) => {
  if (event.button !== 0) return;
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
      const minX = -(boxWidth - visibleEdge);
      const maxX = windowWidth - visibleEdge;
      const minY = 0;
      const maxY = windowHeight - visibleEdge;

      if (newX < minX) newX = minX;
      if (newX > maxX) newX = maxX;
      if (newY < minY) newY = minY;
      if (newY > maxY) newY = maxY;
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
});

onUnmounted(() => {
  window.removeEventListener('resize', clampPosition);

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
    const minX = -(boxWidth - visibleEdge);
    const maxX = windowWidth - visibleEdge;
    const minY = 0;
    const maxY = windowHeight - visibleEdge;

    let newX = x.value;
    let newY = y.value;

    if (newX < minX) newX = minX;
    if (newX > maxX) newX = maxX;
    if (newY < minY) newY = minY;
    if (newY > maxY) newY = maxY;
    
    x.value = newX;
    y.value = newY;
  }
};

const clickCancel = () => {
  emit('cancel');
};
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease-out;
}
.modal-enter-active .modal-dialog,
.modal-leave-active .modal-dialog {
  transition: opacity 0.2s ease-out, transform 0.2s ease-out;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
.modal-enter-from .modal-dialog,
.modal-leave-to .modal-dialog {
  opacity: 0;
  transform: translateY(10px);
}
</style>

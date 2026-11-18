<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="visible" class="fixed inset-0 bg-black/30 z-600">
        <div
          ref="modalDialogRef"
          role="dialog"
          aria-modal="true"
          :aria-label="title"
          tabindex="-1"
          class="modal-dialog outline-none text-base-content/70 bg-base-200/80 backdrop-blur-md border border-base-content/30 rounded-box overflow-hidden flex flex-col"
          :style="{ position: 'fixed', top: y + 'px', left: x + 'px', width: width + 'px', ...(height !== undefined && { height: height + 'px' }) }"
        >
        <!-- title bar -->
        <div
          class="p-3 flex items-center justify-between select-none cursor-default shrink-0"
          @mousedown="dragStart"
        >
          <span class="min-w-0 flex-1">{{ title }}</span>
          <div v-if="$slots['title-actions']" class="mr-2 shrink-0" @mousedown.stop>
            <slot name="title-actions"></slot>
          </div>
          <TButton
            v-if="!hideClose"
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
import { ref, onMounted, onBeforeUnmount, onUnmounted, nextTick } from 'vue';
import { IconClose } from '@/common/icons';
import TButton from '@/components/TButton.vue';

const props = defineProps({
  hideClose: Boolean,
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
  },
  // When provided, dialog position is persisted in localStorage under this key
  // and restored on next mount. Without it, dialog always centers on mount.
  positionKey: {
    type: String,
    default: ''
  }
});

const emit = defineEmits(['cancel']);

const modalDialogRef = ref<HTMLDivElement | null>(null);
const visible = ref(false);
onMounted(() => { visible.value = true; });

let previousFocus: HTMLElement | null = null;

onBeforeUnmount(() => {
  if (modalDialogRef.value?.contains(document.activeElement) && previousFocus?.isConnected) {
    previousFocus.focus({ preventScroll: true });
  }
});

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

const POS_PREFIX = 'lap.dialog.pos.';
// Guards against persisting the initial (0,0) if the dialog unmounts
// before the nextTick positioning pass has run.
let positionReady = false;

const loadPosition = (): boolean => {
  if (!props.positionKey) return false;
  try {
    const raw = localStorage.getItem(POS_PREFIX + props.positionKey);
    if (!raw) return false;
    const parsed = JSON.parse(raw);
    if (typeof parsed?.x !== 'number' || typeof parsed?.y !== 'number') return false;
    if (!Number.isFinite(parsed.x) || !Number.isFinite(parsed.y)) return false;
    x.value = parsed.x;
    y.value = parsed.y;
    return true;
  } catch {
    return false;
  }
};

const savePosition = () => {
  if (!props.positionKey || !positionReady) return;
  try {
    localStorage.setItem(
      POS_PREFIX + props.positionKey,
      JSON.stringify({ x: Math.round(x.value), y: Math.round(y.value) })
    );
  } catch {
    // ignore quota / private-mode errors
  }
};

onMounted(() => {
  previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  window.addEventListener('resize', clampPosition);
  nextTick(() => {
    const dialog = modalDialogRef.value;
    if (!dialog) return;
    if (!dialog.contains(document.activeElement)) dialog.focus({ preventScroll: true });
    // Restore saved position if available, otherwise center.
    // Either way, clamp to viewport to handle resolution / display changes.
    if (!loadPosition()) {
      centerDialog();
    } else {
      clampPosition();
    }
    positionReady = true;
  });
});

onUnmounted(() => {
  window.removeEventListener('resize', clampPosition);

  // In case component is unmounted while dragging
  if (isDragging.value) {
    dragEnd();
  }

  // Persist final position once per dialog session (on close), not per drag.
  savePosition();
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
  transform: translateY(4px);
}
</style>

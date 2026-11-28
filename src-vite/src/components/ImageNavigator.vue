<template>
  <div ref="container" data-image-navigator class="relative overflow-hidden select-none"
    :class="viewport?.pannable ? (isDragging ? 'cursor-grabbing' : 'cursor-grab') : 'cursor-pointer'"
    :style="{ touchAction: viewport?.pannable ? 'none' : 'auto' }"
    @pointerdown.stop="startDrag" @pointermove.stop="moveDrag"
    @pointerup.stop="endDrag" @pointercancel.stop="cancelDrag" @lostpointercapture="cancelDrag"
    @click.stop @dblclick.stop.prevent="emit('toggle-fit')" @wheel.stop.prevent="zoom">
    <div class="absolute" :style="contentStyle">
      <img :src="source" :style="imageStyle" draggable="false" class="absolute max-w-none pointer-events-none" />
      <SharpnessGrid :source="source" :enabled="sharpness" :width="layout.width" :height="layout.height" :rotate="rotation" />
      <div v-if="viewport?.pannable" class="absolute border-2 border-primary cursor-move pointer-events-none"
        :style="boxStyle" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import SharpnessGrid from '@/components/SharpnessGrid.vue';
import { navigatorLayout, navigatorPoint, navigatorBox } from '@/common/imageNavigator';
import type { NavigatorViewport } from '@/common/imageNavigator';

const props = defineProps<{ source: string; viewport: NavigatorViewport; sharpness: boolean }>();
const emit = defineEmits<{
  navigate: [point: { normX: number; normY: number }];
  zoom: [factor: number];
  'toggle-fit': [];
  dragging: [value: boolean];
}>();
const container = ref<HTMLElement | null>(null);
const isDragging = ref(false);
const size = ref({ width: 0, height: 0 });
const rotation = computed(() => props.viewport?.rotate || 0);
const layout = computed(() => navigatorLayout(props.viewport, size.value));
const contentStyle = computed(() => ({ left: `${layout.value.left}px`, top: `${layout.value.top}px`,
  width: `${layout.value.width}px`, height: `${layout.value.height}px` }));
const imageStyle = computed(() => ({ width: `${layout.value.imageWidth}px`, height: `${layout.value.imageHeight}px`,
  left: `${(layout.value.width - layout.value.imageWidth) / 2}px`,
  top: `${(layout.value.height - layout.value.imageHeight) / 2}px`, transform: `rotate(${rotation.value}deg)` }));
const boxStyle = computed(() => {
  const box = navigatorBox(props.viewport, layout.value);
  return { left: `${box.left}px`, top: `${box.top}px`, width: `${box.width}px`, height: `${box.height}px`,
    boxShadow: '0 0 0 9999px color-mix(in srgb, var(--color-base-200) 30%, transparent)' };
});
let observer: ResizeObserver | null = null;
onMounted(() => {
  observer = new ResizeObserver(([entry]) => { size.value = { width: entry.contentRect.width, height: entry.contentRect.height }; });
  if (container.value) observer.observe(container.value);
});
onBeforeUnmount(() => { observer?.disconnect(); cancelDrag(); });
let drag: { id: number; x: number; y: number; normX: number; normY: number; moved: boolean; inBox: boolean } | null = null;
function point(event: PointerEvent) {
  const rect = container.value!.getBoundingClientRect();
  return navigatorPoint(props.viewport, layout.value, event.clientX - rect.left, event.clientY - rect.top);
}
function startDrag(event: PointerEvent) {
  if (event.button !== 0 || !props.viewport?.pannable || !container.value) return;
  event.preventDefault();
  const rect = container.value.getBoundingClientRect();
  const box = navigatorBox(props.viewport, layout.value);
  const x = event.clientX - rect.left - layout.value.left;
  const y = event.clientY - rect.top - layout.value.top;
  drag = { id: event.pointerId, x: event.clientX, y: event.clientY,
    normX: props.viewport.normX, normY: props.viewport.normY, moved: false,
    inBox: x >= box.left && x <= box.left + box.width && y >= box.top && y <= box.top + box.height };
  container.value.setPointerCapture(event.pointerId);
  isDragging.value = true;
  emit('dragging', true);
}
function moveDrag(event: PointerEvent) {
  if (!drag || drag.id !== event.pointerId) return;
  event.preventDefault();
  const dx = event.clientX - drag.x;
  const dy = event.clientY - drag.y;
  if (!drag.moved && Math.hypot(dx, dy) < 3) return;
  drag.moved = true;
  if (!drag.inBox) { emit('navigate', point(event)); return; }
  const angle = rotation.value * Math.PI / 180;
  emit('navigate', {
    normX: drag.normX + (dx * Math.cos(angle) + dy * Math.sin(angle)) / layout.value.imageWidth,
    normY: drag.normY + (-dx * Math.sin(angle) + dy * Math.cos(angle)) / layout.value.imageHeight,
  });
}
function endDrag(event: PointerEvent) {
  if (!drag || drag.id !== event.pointerId) return;
  if (!drag.moved) emit('navigate', point(event));
  cancelDrag();
}
function cancelDrag() {
  const id = drag?.id;
  drag = null;
  isDragging.value = false;
  if (id !== undefined && container.value?.hasPointerCapture(id)) container.value.releasePointerCapture(id);
  emit('dragging', false);
}
watch(() => props.source, cancelDrag);
function zoom(event: WheelEvent) {
  const delta = event.deltaY * (event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? size.value.height : 1);
  if (delta) emit('zoom', Math.exp(-Math.max(-100, Math.min(100, delta)) / (event.ctrlKey ? 96 : 500)));
}
</script>

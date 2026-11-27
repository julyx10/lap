<template>
  <button
    ref="trigger"
    type="button"
    class="inline-flex size-5 shrink-0 items-center justify-center rounded text-base-content/40 hover:text-base-content focus-visible:text-base-content focus-visible:outline-2 focus-visible:outline-primary"
    :aria-label="label"
    :aria-describedby="open ? tooltipId : undefined"
    @mouseenter="enter"
    @mouseleave="leave"
    @focus="focused = true"
    @blur="focused = false"
    @click.stop="toggle"
  >
    <IconInformation class="size-3.5" aria-hidden="true" />
  </button>
  <Teleport to="body">
    <div
      v-if="open"
      :id="tooltipId"
      ref="tooltip"
      role="tooltip"
      class="fixed z-1000 w-75 max-w-[calc(100vw-16px)] rounded-box border border-base-content/10 bg-base-100 px-3 py-2 text-xs leading-5 text-base-content shadow-lg"
      :style="position"
      @mouseenter="enter"
      @mouseleave="leave"
    >{{ text }}</div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, useId, watch } from 'vue';
import { IconInformation } from '@/common/icons';

defineProps<{ label: string; text: string }>();
const tooltipId = useId();
const trigger = ref<HTMLButtonElement | null>(null);
const tooltip = ref<HTMLElement | null>(null);
const hovered = ref(false);
const focused = ref(false);
const pinned = ref(false);
const dismissed = ref(false);
const open = computed(() => !dismissed.value && (hovered.value || focused.value || pinned.value));
const position = ref({ left: '0px', top: '0px', visibility: 'hidden' as 'hidden' | 'visible' });
let hoverTimer: ReturnType<typeof setTimeout> | undefined;

function enter() {
  clearTimeout(hoverTimer);
  hovered.value = true;
}

function leave() {
  clearTimeout(hoverTimer);
  hoverTimer = setTimeout(() => { hovered.value = false; }, 150);
}

function close() {
  pinned.value = false;
  dismissed.value = true;
}

function toggle() {
  if (pinned.value) close();
  else {
    dismissed.value = false;
    pinned.value = true;
  }
}

async function updatePosition() {
  await nextTick();
  if (!trigger.value || !tooltip.value) return;
  const anchor = trigger.value.getBoundingClientRect();
  const box = tooltip.value.getBoundingClientRect();
  const left = Math.max(8, Math.min(anchor.left + (anchor.width - box.width) / 2, window.innerWidth - box.width - 8));
  let top = anchor.top - box.height - 6;
  if (top < 8) top = anchor.bottom + 6;
  top = Math.max(8, Math.min(top, window.innerHeight - box.height - 8));
  position.value = { left: `${Math.round(left)}px`, top: `${Math.round(top)}px`, visibility: 'visible' };
}

function onPointerDown(event: PointerEvent) {
  const target = event.target as Node;
  if (!trigger.value?.contains(target) && !tooltip.value?.contains(target)) close();
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.stopPropagation();
    close();
  }
}

function removeListeners() {
  window.removeEventListener('resize', updatePosition);
  window.removeEventListener('scroll', updatePosition, true);
  window.removeEventListener('pointerdown', onPointerDown, true);
  window.removeEventListener('keydown', onKeyDown, true);
}

watch([hovered, focused], ([hover, focus]) => {
  if (!hover && !focus) dismissed.value = false;
});
watch(open, (visible) => {
  removeListeners();
  if (!visible) return;
  position.value.visibility = 'hidden';
  void updatePosition();
  window.addEventListener('resize', updatePosition);
  window.addEventListener('scroll', updatePosition, true);
  window.addEventListener('pointerdown', onPointerDown, true);
  window.addEventListener('keydown', onKeyDown, true);
});
onBeforeUnmount(() => {
  clearTimeout(hoverTimer);
  removeListeners();
});
</script>

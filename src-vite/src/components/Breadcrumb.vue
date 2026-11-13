<template>
  <div class="group/field flex items-center gap-1 min-w-0 min-h-6">
    <component
      :is="icon"
      v-if="icon"
      :class="[
        'w-3.5 h-3.5 shrink-0',
        disabled ? 'text-base-content/70' : 'text-base-content/70'
      ]"
    />
    <div ref="containerRef" class="grid min-w-0 flex-1 overflow-hidden">
      <div
        class="col-start-1 row-start-1 flex min-w-0 items-center whitespace-nowrap"
        :class="[sizeClass, disabled ? 'text-base-content/70' : (highlight ? 'text-primary' : 'text-base-content/70')]"
      >
        <template v-if="startIndex > 0">
          <span class="shrink-0">…</span>
          <span class="mx-1 shrink-0 text-base-content/30">&gt;</span>
        </template>
        <template
          v-for="(item, idx) in visibleItems"
          :key="item.path"
        >
          <span
            v-if="idx > 0"
            class="mx-1 text-base-content/30"
          >&gt;</span>
          <button
            v-if="!disabled"
            type="button"
            :class="[
              'cursor-pointer transition-colors hover:text-base-content',
              shouldTruncateVisibleItem ? 'min-w-0 truncate' : 'shrink-0'
            ]"
            @click.stop="$emit('navigate', item.path)"
          >{{ item.label }}</button>
          <span
            v-else
            :class="shouldTruncateVisibleItem ? 'min-w-0 truncate' : 'shrink-0'"
          >{{ item.label }}</span>
        </template>
      </div>
      <div
        ref="measureRef"
        class="pointer-events-none invisible col-start-1 row-start-1 flex items-center whitespace-nowrap"
        :class="sizeClass"
        aria-hidden="true"
      >
        <span data-breadcrumb-ellipsis>…<span class="mx-1">&gt;</span></span>
        <span data-breadcrumb-separator class="mx-1">&gt;</span>
        <span
          v-for="(item, idx) in items"
          :key="`measure-${item.path}`"
          data-breadcrumb-measure
          class="shrink-0"
        ><span v-if="idx > 0" class="mx-1">&gt;</span>{{ item.label }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';

export interface BreadcrumbItem {
  label: string;
  path: string;
}

const props = withDefaults(defineProps<{
  items: BreadcrumbItem[];
  icon?: any;
  disabled?: boolean;
  highlight?: boolean;
  size?: 'small' | 'medium' | 'large';
}>(), {
  icon: undefined,
  disabled: false,
  highlight: false,
  size: 'medium',
});

defineEmits<{
  navigate: [path: string];
}>();

const containerRef = ref<HTMLElement | null>(null);
const measureRef = ref<HTMLElement | null>(null);
const startIndex = ref(0);
const visibleItems = computed(() =>
  props.items.slice(startIndex.value)
);
const sizeClass = computed(() => ({
  small: 'text-xs',
  medium: 'text-sm',
  large: 'text-base',
}[props.size]));
const shouldTruncateVisibleItem = computed(() =>
  props.items.length === 1 || (startIndex.value > 0 && visibleItems.value.length === 1)
);
let resizeObserver: ResizeObserver | null = null;

async function updateVisibility() {
  await nextTick();
  const container = containerRef.value;
  const measure = measureRef.value;
  if (!container || !measure || props.items.length === 0) {
    startIndex.value = 0;
    return;
  }
  if (props.items.length === 1) {
    startIndex.value = 0;
    return;
  }

  const itemWidths = Array.from(
    measure.querySelectorAll<HTMLElement>('[data-breadcrumb-measure]')
  ).map(item => item.offsetWidth);
  const totalWidth = itemWidths.reduce((sum, width) => sum + width, 0);
  if (totalWidth <= container.clientWidth) {
    startIndex.value = 0;
    return;
  }

  const ellipsisWidth = measure.querySelector<HTMLElement>('[data-breadcrumb-ellipsis]')?.offsetWidth || 0;
  const separatorEl = measure.querySelector<HTMLElement>('[data-breadcrumb-separator]');
  const separatorWidth = separatorEl?.offsetWidth || 0;
  let usedWidth = ellipsisWidth;
  let newStartIndex = itemWidths.length - 1;
  for (let index = itemWidths.length - 1; index >= 0; index--) {
    const itemWidth = itemWidths[index] - (index === itemWidths.length - 1 ? separatorWidth : 0);
    if (index < itemWidths.length - 1 && usedWidth + itemWidth > container.clientWidth) {
      break;
    }
    usedWidth += itemWidth;
    newStartIndex = index;
  }
  startIndex.value = newStartIndex;
}

watch(containerRef, (container) => {
  resizeObserver?.disconnect();
  resizeObserver = null;
  if (container) {
    resizeObserver = new ResizeObserver(updateVisibility);
    resizeObserver.observe(container);
  }
  updateVisibility();
});

watch(() => props.items, updateVisibility, { flush: 'post' });

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
});
</script>

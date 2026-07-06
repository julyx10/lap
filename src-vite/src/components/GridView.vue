<template>
  <div
    ref="containerRef"
    class="w-full h-full"
    :class="{ 
      'pointer-events-none': uiStore.inputStack.length > 0,
    }"
    @wheel="onWheel"
    @dragstart.capture.prevent
  >
    <VirtualScroll
      v-if="fileList.length > 0"
      ref="scroller"
      class="w-full h-full no-scrollbar"
      :class="{
        'pt-12': !config.settings.grid.showFilmStrip,
        'pb-8': !config.settings.grid.showFilmStrip && config.settings.showStatusBar,
        'pb-1': !config.settings.grid.showFilmStrip && !config.settings.showStatusBar,
      }"
      :items="renderItems"
      :direction="config.settings.grid.showFilmStrip && config.settings.grid.previewPosition < 2 ? 'horizontal' : 'vertical'"
      :grid-items="config.settings.grid.showFilmStrip ? 1 : columnCount"
      :item-size="config.settings.grid.showFilmStrip ? (config.settings.grid.previewPosition < 2 ? filmStripItemSize : itemHeight) : itemHeight"
      :item-secondary-size="!config.settings.grid.showFilmStrip ? itemWidth : (config.settings.grid.previewPosition >= 2 ? itemWidth : undefined)"
      :key="`${config.settings.grid.showFilmStrip}`"
      :geometry="virtualScrollGeometry"
      :content-height="virtualScrollContentHeight"
      :transition="isLayoutTransitioning"
      key-field="id"
      :emit-update="true"
      :buffer="4"
      v-slot="{ item, index }"
      @update="onUpdate"
      @scroll="onScroll"
    >
      <div
        v-if="isGroupRow(item)"
        class="w-full h-12 flex items-center gap-1 px-2 text-base-content/70 select-none bg-base-100/5 rounded-box"
      >
        <input
          v-if="selectMode"
          type="checkbox"
          class="checkbox checkbox-xs border-base-content/30"
          :checked="getGroupSelectionState(item).checked"
          :indeterminate.prop="getGroupSelectionState(item).indeterminate"
          :disabled="isGroupSelectionLoading(item)"
          @change="(event) => $emit('group-select-toggled', item, (event.target as HTMLInputElement).checked)"
        />
        <span v-if="selectMode && isGroupSelectionLoading(item)" class="loading loading-spinner loading-xs text-primary"></span>
        <component :is="getGroupIcon(item)" class="w-4 h-4 shrink-0 text-base-content/30" />
        <Breadcrumb
          v-if="isFolderPathGroup()"
          :items="getFolderGroupBreadcrumbItems(item)"
          disabled
          class="min-w-0 flex-1 overflow-hidden"
        />
        <span v-else class="min-w-0 flex-1 truncate text-sm text-base-content/30">{{ item.label }}</span>
        <span class="badge badge-xs shrink-0 text-xs bg-base-100 text-base-content/30">
          {{ item.countLabel || Number(item.count || 0).toLocaleString() }}
        </span>
      </div>
      <div
        v-else
        class="w-full h-full flex items-center justify-center overflow-hidden"
        @pointerdown="onItemPointerDown($event, getFileIndex(item, index), getFileItem(item))"
      >
        <Thumbnail
          v-if="getFileItem(item) && !getFileItem(item).isPlaceholder"
          :key="getThumbnailKey(item, index)"
          :id="'item-' + getFileIndex(item, index)"
          :file="getFileItem(item)"
          :is-selected="selectMode ? Boolean(getFileItem(item).isSelected) : getFileIndex(item, index) === selectedItemIndex"
          :is-active="getFileIndex(item, index) === selectedItemIndex"
          :select-mode="selectMode"
          :query-source="querySource"
          :dedup-status="dedupStatuses[Number(getFileItem(item).id)]"
          @clicked="(modifiers) => $emit('item-clicked', getFileIndex(item, index), modifiers)"
          @dblclicked="(modifiers) => $emit('item-dblclicked', getFileIndex(item, index), modifiers)"
          @select-toggled="(shiftKey) => $emit('item-select-toggled', getFileIndex(item, index), shiftKey)"
          @action="(actionName) => $emit('item-action', { action: actionName, index: getFileIndex(item, index) })"
          @select-contextmenu="(pos) => $emit('item-select-contextmenu', { ...pos, index: getFileIndex(item, index) })"
        />
        <div v-else class="w-full h-full bg-base-200/70"></div>
      </div>
    </VirtualScroll>
    <!-- Empty State / Loading -->
    <div v-if="fileList.length === 0" class="absolute inset-0 flex flex-col items-center justify-center">
      <div class="text-base-content/30 flex flex-col items-center gap-2 text-center px-4">
        <Transition name="fade" appear>
          <div v-if="showDelayedLoading" class="flex flex-col items-center gap-2">
            <span class="loading loading-dots loading-lg text-primary"></span>
            <span>{{ $t('tooltip.loading') }}</span>
          </div>
        </Transition>
        <template v-if="contentReady">
          <template v-if="showFolderFiles && folderExcluded">
            <span>{{ $t('tooltip.not_found.folder_excluded') }}</span>
            <span class="text-xs">{{ $t('tooltip.not_found.folder_excluded_hint') }}</span>
          </template>
          <template v-else-if="showFolderFiles">
            <span>{{ $t('tooltip.not_found.folder_files') }}</span>
            <span class="text-xs">{{ $t('tooltip.not_found.folder_files_hint') }}</span>
          </template>
          <span v-else class="whitespace-pre-line">{{ emptyMessage || $t('tooltip.not_found.files') }}</span>
        </template>
      </div>
    </div>

  </div>

</template>

<script setup lang="ts">

import { watch, ref, onMounted, onBeforeUnmount, computed, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import Thumbnail from '@/components/Thumbnail.vue';
import VirtualScroll from '@/components/VirtualScroll.vue';
import { GROUP } from '@/common/constants';
import { calculateJustifiedLayout, calculateLinearRowLayout, calculateLinearColumnLayout, calculateMasonryLayout, type Geometry } from '@/common/layout';
import { buildFolderBreadcrumbs, formatFolderBreadcrumb, isWithinRootPath } from '@/common/utils';
import Breadcrumb from '@/components/Breadcrumb.vue';
import {
  IconCalendarDay,
  IconCalendarMonth,
  IconCameraAperture,
  IconFile,
  IconFiles,
  IconFolder,
  IconLocation,
  IconStarFilled,
} from '@/common/icons';

const props = withDefaults(defineProps<{
  selectedItemIndex: number;
  fileList: any[];
  timelineData?: any[];
  sortType?: number;
  showFolderFiles?: boolean;
  folderExcluded?: boolean;
  selectMode?: boolean;
  contentReady?: boolean;
  emptyMessage?: string;
  layoutVersion?: number;
  groupBy?: number;
  groupSelectedCounts?: Record<string, number>;
  groupSelectionLoading?: Record<string, boolean>;
  folderGroupRoots?: Array<{ path: string; name?: string }>;
  querySource?: string;
  dedupStatuses?: Record<number, 'keep' | 'dup'>;
}>(), {
  selectedItemIndex: -1,
  timelineData: () => [],
  sortType: 0,
  showFolderFiles: false,
  folderExcluded: false,
  selectMode: false,
  contentReady: false,
  emptyMessage: '',
  layoutVersion: 0,
  groupBy: 0,
  groupSelectedCounts: () => ({}),
  groupSelectionLoading: () => ({}),
  folderGroupRoots: () => [],
  querySource: '',
  dedupStatuses: () => ({}),
});

const emit = defineEmits([
  'item-clicked',
  'item-dblclicked',
  'item-select-toggled',
  'item-action',
  'item-select-contextmenu',
  'date-group-select',
  'group-select-toggled',
  'request-scroll',
  'visible-range-update',
  'scroll',
  'layout-update',
  'item-drag-start',
  'item-drag',
  'item-drag-end',
]);

const uiStore = useUIStore();
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const containerRef = ref<HTMLElement | null>(null);
const scroller = ref<any>(null);
const columnCount = ref(4);
const containerWidth = ref(0);
const groupHeaderHeight = computed(() => 48 * Number(config.settings.scale || 1));
let pendingPointerDrag: {
  pointerId: number;
  index: number;
  startX: number;
  startY: number;
  hotspotXRatio: number;
  hotspotYRatio: number;
  active: boolean;
} | null = null;

function clearPointerDragListeners() {
  document.removeEventListener('pointermove', onDocumentPointerMove, true);
  document.removeEventListener('pointerup', onDocumentPointerUp, true);
  document.removeEventListener('pointercancel', onDocumentPointerUp, true);
}

function onItemPointerDown(event: PointerEvent, index: number, file: any) {
  if (index < 0) return;
  const target = event.target as HTMLElement;
  if (
    event.button !== 0
    || event.pointerType === 'touch'
    || target.closest('button, input, a, [role="button"]')
    || !file
    || file.isPlaceholder
  ) return;
  const itemElement = document.getElementById(`item-${index}`);
  const itemRect = itemElement?.getBoundingClientRect();
  pendingPointerDrag = {
    pointerId: event.pointerId,
    index,
    startX: event.clientX,
    startY: event.clientY,
    hotspotXRatio: itemRect?.width
      ? Math.max(0, Math.min(1, (event.clientX - itemRect.left) / itemRect.width))
      : 0.5,
    hotspotYRatio: itemRect?.height
      ? Math.max(0, Math.min(1, (event.clientY - itemRect.top) / itemRect.height))
      : 0.5,
    active: false,
  };
  document.addEventListener('pointermove', onDocumentPointerMove, true);
  document.addEventListener('pointerup', onDocumentPointerUp, true);
  document.addEventListener('pointercancel', onDocumentPointerUp, true);
}

function onDocumentPointerMove(event: PointerEvent) {
  const drag = pendingPointerDrag;
  if (!drag || event.pointerId !== drag.pointerId) return;
  if (!drag.active) {
    if (Math.hypot(event.clientX - drag.startX, event.clientY - drag.startY) < 6) return;
    drag.active = true;
    document.documentElement.style.userSelect = 'none';
    document.documentElement.style.webkitUserSelect = 'none';
    emit('item-drag-start', {
      event,
      index: drag.index,
      hotspotXRatio: drag.hotspotXRatio,
      hotspotYRatio: drag.hotspotYRatio,
    });
  }
  event.preventDefault();
  emit('item-drag', event);
}

function onDocumentPointerUp(event: PointerEvent) {
  const drag = pendingPointerDrag;
  if (!drag || event.pointerId !== drag.pointerId) return;
  if (drag.active) {
    event.preventDefault();
    event.stopPropagation();
    emit('item-drag-end', event);
  }
  pendingPointerDrag = null;
  document.documentElement.style.userSelect = '';
  document.documentElement.style.webkitUserSelect = '';
  clearPointerDragListeners();
}

function isGeometryGridStyle(style: number) {
  return style === 2 || style === 3;
}

const renderItems = computed(() => props.fileList);
const hasGroupRows = computed(() =>
  Number(props.groupBy || 0) > 0 || isGroupRow(renderItems.value[0])
);
const fileIndexToRowIndex = computed(() => {
  const map = new Map<number, number>();
  if (!hasGroupRows.value) return map;
  renderItems.value.forEach((item, rowIndex) => {
    if (!isGroupRow(item)) {
      const fileIndex = getFileIndex(item, rowIndex);
      if (fileIndex >= 0) map.set(fileIndex, rowIndex);
    }
  });
  return map;
});
const rowIndexToFileIndex = computed(() => {
  const map = new Map<number, number>();
  if (!hasGroupRows.value) return map;
  renderItems.value.forEach((item, rowIndex) => {
    if (!isGroupRow(item)) {
      const fileIndex = getFileIndex(item, rowIndex);
      if (fileIndex >= 0) map.set(rowIndex, fileIndex);
    }
  });
  return map;
});

const groupedLayoutGeometryResult = computed(() => {
  if (!hasGroupRows.value || renderItems.value.length === 0 || containerWidth.value <= 0) {
    return { boxes: [], contentSize: 0 };
  }

  const { style, size } = config.settings.grid;
  const boxes: Geometry[] = new Array(renderItems.value.length);

  if (!isGeometryGridStyle(style)) {
    let y = 0;
    let col = 0;

    renderItems.value.forEach((item, rowIndex) => {
      if (isGroupRow(item)) {
        if (col > 0) {
          y += itemHeight.value;
          col = 0;
        }
        boxes[rowIndex] = { x: 0, y, width: containerWidth.value, height: groupHeaderHeight.value };
        y += groupHeaderHeight.value;
        return;
      }

      boxes[rowIndex] = {
        x: col * itemWidth.value,
        y,
        width: itemWidth.value,
        height: itemHeight.value,
      };
      col += 1;
      if (col >= columnCount.value) {
        y += itemHeight.value;
        col = 0;
      }
    });

    if (col > 0) y += itemHeight.value;
    return { boxes, contentSize: y };
  }

  let y = 0;
  let groupFiles: any[] = [];
  let groupRowIndices: number[] = [];

  const flushGroup = () => {
    if (groupFiles.length === 0) return;
    const result = style === 3
      ? calculateMasonryLayout(groupFiles, containerWidth.value, size, 0)
      : calculateJustifiedLayout(groupFiles, containerWidth.value, size, 0);
    result.boxes.forEach((box, index) => {
      boxes[groupRowIndices[index]] = {
        ...box,
        y: box.y + y,
      };
    });
    y += result.containerHeight;
    groupFiles = [];
    groupRowIndices = [];
  };

  renderItems.value.forEach((item, rowIndex) => {
    if (isGroupRow(item)) {
      flushGroup();
      boxes[rowIndex] = { x: 0, y, width: containerWidth.value, height: groupHeaderHeight.value };
      y += groupHeaderHeight.value;
      return;
    }
    groupFiles.push(getFileItem(item));
    groupRowIndices.push(rowIndex);
  });
  flushGroup();

  return { boxes, contentSize: y };
});

const layoutGeometryResult = computed(() => {
  if (props.fileList.length === 0) {
    return { boxes: [], contentSize: 0 };
  }

  const { style, size, showFilmStrip } = config.settings.grid;

  if (hasGroupRows.value) {
    return groupedLayoutGeometryResult.value;
  }

  if (showFilmStrip) {
    if (isGeometryGridStyle(style)) {
      const isVertical = config.settings.grid.previewPosition >= 2;
      if (isVertical) {
        if (containerWidth.value <= 0) return { boxes: [], contentSize: 0 };
        const result = calculateLinearColumnLayout(props.fileList, containerWidth.value, 0);
        return { boxes: result.boxes, contentSize: result.containerHeight };
      }
      const result = calculateLinearRowLayout(props.fileList, size, 0);
      return { boxes: result.boxes, contentSize: result.containerWidth };
    }
  } else {
    if (style === 2 && containerWidth.value > 0) {
      const result = calculateJustifiedLayout(props.fileList, containerWidth.value, size, 0);
      return { boxes: result.boxes, contentSize: result.containerHeight };
    }
    else if (style === 3 && containerWidth.value > 0) {
      const result = calculateMasonryLayout(props.fileList, containerWidth.value, size, 0);
      return { boxes: result.boxes, contentSize: result.containerHeight };
    }
  }
  return { boxes: [], contentSize: 0 };
});

const layoutGeometry = computed(() => layoutGeometryResult.value.boxes);
const layoutContentHeight = computed(() => layoutGeometryResult.value.contentSize);
const usesGeometryLayout = computed(() =>
  hasGroupRows.value ||
  isGeometryGridStyle(config.settings.grid.style)
);
const virtualScrollGeometry = computed(() =>
  usesGeometryLayout.value ? layoutGeometry.value : undefined
);
const virtualScrollContentHeight = computed(() =>
  usesGeometryLayout.value ? layoutContentHeight.value : undefined
);
const isLayoutTransitioning = ref(false);
const startGridSize = ref(0);
let layoutTransitionTimer: ReturnType<typeof setTimeout> | null = null;
let layoutAnchorVersion = 0;
let isInitialLayout = true;

const gap = 8; // Gap between items
const isVerticalFilmstrip = computed(() => config.settings.grid.showFilmStrip && config.settings.grid.previewPosition >= 2);

// item width and height(including gap)
const itemWidth = computed(() => {
  const { style, size } = config.settings.grid;
  if (isVerticalFilmstrip.value && containerWidth.value > 0) {
    return containerWidth.value;
  }
  if (style === 0) return size + 20; // size + padding(4*2) + border(2*2) + gap(8)
  return size;
});

const itemHeight = computed(() => {
  const { style, size } = config.settings.grid;
  
  if (style === 0) {
    let labelHeight = 0;
    if (config.settings.grid.labelPrimary > 0) labelHeight += 18;   // text-sm
    if (config.settings.grid.labelSecondary > 0) labelHeight += 16; // text-xs
    
    if (isVerticalFilmstrip.value && containerWidth.value > 0) {
      return containerWidth.value + 12 + labelHeight; // Narrower padding in filmstrip
    }
    return size + 20 + labelHeight; // size + padding/border/gap(20) + labels
  }
  if (style === 1) return itemWidth.value + gap * 0.5;
  
  if (isVerticalFilmstrip.value && containerWidth.value > 0) {
    return containerWidth.value;
  }
  return size;
});

const filmStripItemSize = computed(() => {
  return itemWidth.value;
});

let resizeObserver: ResizeObserver | null = null;
const showDelayedLoading = ref(false);
let loadingDelayTimer: ReturnType<typeof setTimeout> | null = null;

function updateColumnCount() {
  if (containerRef.value) {
    containerWidth.value = containerRef.value.clientWidth;
    if (itemWidth.value > 0) {
      columnCount.value = Math.max(1, Math.floor(containerWidth.value / itemWidth.value));
    }
  }
}

function updateLayout() {
  updateColumnCount();
  emit('layout-update', { height: layoutContentHeight.value });
}

watch(() => [config.settings.grid.size, config.settings.grid.style, config.settings.grid.showFilmStrip], async () => {
  if (isInitialLayout) {
    isInitialLayout = false;
    updateColumnCount();
    return;
  }

  const anchorVersion = ++layoutAnchorVersion;
  isLayoutTransitioning.value = true;
  updateColumnCount();

  await nextTick();
  if (anchorVersion !== layoutAnchorVersion) return;

  centerItem(props.selectedItemIndex);

  if (layoutTransitionTimer) clearTimeout(layoutTransitionTimer);
  layoutTransitionTimer = setTimeout(() => {
    layoutTransitionTimer = null;
    isLayoutTransitioning.value = false;
  }, 500);
});

watch(() => props.fileList, () => {
  updateLayout();
});

watch(() => props.timelineData, () => {
  updateLayout();
});

watch(() => props.layoutVersion, () => {
  updateLayout();
});

watch(
  () => props.contentReady,
  (ready) => {
    if (loadingDelayTimer) {
      clearTimeout(loadingDelayTimer);
      loadingDelayTimer = null;
    }

    if (ready) {
      showDelayedLoading.value = false;
      return;
    }

    showDelayedLoading.value = false;
    loadingDelayTimer = setTimeout(() => {
      loadingDelayTimer = null;
      if (!props.contentReady) {
        showDelayedLoading.value = true;
      }
    }, 500);
  },
  { immediate: true }
);

watch(layoutContentHeight, (newHeight) => {
  emit('layout-update', { height: newHeight });
});

watch(() => props.selectedItemIndex, (newValue) => {
  if (newValue !== -1) {
    scrollToItem(newValue);
  }
});

onMounted(() => {
  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      // updateColumnCount(); // merged into updateLayout
      updateLayout();
      if (props.selectedItemIndex !== -1) {
        scrollToItem(props.selectedItemIndex);
      }
    });
    resizeObserver.observe(containerRef.value);
    updateLayout();

    // gesture events for macOS touchpad pinch
    containerRef.value.addEventListener('gesturestart', onGestureStart as any);
    containerRef.value.addEventListener('gesturechange', onGestureChange as any);
  }
  window.addEventListener('keydown', onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown);
  clearPointerDragListeners();
  pendingPointerDrag = null;
  document.documentElement.style.userSelect = '';
  document.documentElement.style.webkitUserSelect = '';
  if (loadingDelayTimer) {
    clearTimeout(loadingDelayTimer);
    loadingDelayTimer = null;
  }
  if (layoutTransitionTimer) {
    clearTimeout(layoutTransitionTimer);
    layoutTransitionTimer = null;
  }
  if (containerRef.value) {
    containerRef.value.removeEventListener('gesturestart', onGestureStart as any);
    containerRef.value.removeEventListener('gesturechange', onGestureChange as any);
  }
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});

function onGestureStart(e: any) {
  e.preventDefault();
  startGridSize.value = config.settings.grid.size;
}

function onGestureChange(e: any) {
  e.preventDefault();
  if (startGridSize.value > 0) {
    let newSize = Math.round(startGridSize.value * e.scale);
    // Clamp between 120 and 360
    newSize = Math.max(120, Math.min(360, newSize));
    config.settings.grid.size = newSize;
  }
}

function onUpdate(startIndex: number, endIndex: number) {
  emit('visible-range-update', { startIndex, endIndex });
}

function onScroll(e: Event) {
  emit('scroll', e);
}

function onWheel(e: WheelEvent) {
  if (config.settings.grid.showFilmStrip && scroller.value) {
    const isHorizontal = config.settings.grid.previewPosition < 2;
    if (isHorizontal) {
      // If it's a vertical scroll (deltaY) and no horizontal scroll (deltaX),
      // translate it to horizontal scroll
      if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
        scroller.value.$el.scrollLeft += e.deltaY;
        e.preventDefault(); // Prevent default vertical scrolling behavior if any
      }
    }
  }
}

function onKeyDown(e: KeyboardEvent) {
  // Prevent default scrolling for arrow keys and spacebar
  if (['ArrowUp', 'ArrowDown', 'Space', ' '].includes(e.key)) {
    // Allow default behavior if typing in an input
    const target = e.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
      return;
    }
    e.preventDefault();
  }
}

function scrollToItem(index: number, center = false) {
  if (!scroller.value) return;
  
  const el = scroller.value.$el;
  const displayIndex = fileIndexToRowIndex.value.get(index) ?? index;

  const renderedItem = center ? containerRef.value?.querySelector(`#item-${index}`) : null;
  if (renderedItem && !virtualScrollGeometry.value) {
    renderedItem.scrollIntoView({
      behavior: 'auto',
      block: 'center',
      inline: 'center',
    });
    return;
  }
  
  if (!config.settings.grid.showFilmStrip) {
    let itemTop = 0;
    let itemBottom = 0;

    if (virtualScrollGeometry.value && layoutGeometry.value[displayIndex]) {
      const box = layoutGeometry.value[displayIndex];
      itemTop = box.y;
      itemBottom = box.y + box.height;
    } else {
      // Normal Grid Logic
      const row = Math.floor(displayIndex / columnCount.value);
      itemTop = row * itemHeight.value;
      itemBottom = itemTop + itemHeight.value;
    }

    const scrollTop = el.scrollTop;
    const clientHeight = el.clientHeight;
    
    // Account for top and bottom padding
    const topPadding = 48 * Number(config.settings.scale || 1);
    const bottomPadding = config.settings.showStatusBar ? 32 : 4; // pb-8 = 32px, pb-1 = 4px

    if (center) {
      el.scrollTop = Math.min(
        Math.max(0, topPadding + (itemTop + itemBottom) / 2 - clientHeight / 2),
        Math.max(0, el.scrollHeight - clientHeight),
      );
      return;
    }
    
    const viewportTop = scrollTop;
    const viewportBottom = scrollTop + clientHeight - (topPadding + bottomPadding);

    // Only scroll if the item is not fully visible
    const isFullyVisible = itemTop >= viewportTop && itemBottom <= viewportBottom;
    
    if (!isFullyVisible) {
      if (itemTop < viewportTop) {
        // Item is above viewport, scroll to show it at the top
        el.scrollTop = itemTop;
      } else if (itemBottom > viewportBottom) {
        // Item is below viewport, scroll to show it at the bottom (accounting for bottom padding)
        el.scrollTop = itemBottom - clientHeight + (topPadding + bottomPadding);
      }
    }
  } else {
    // Filmstrip mode: center the item
    const isHorizontal = config.settings.grid.previewPosition < 2;
    let itemPos = 0;
    let itemSizeValue = 0;

    if (layoutGeometry.value[displayIndex]) {
      const box = layoutGeometry.value[displayIndex];
      itemPos = isHorizontal ? box.x : box.y;
      itemSizeValue = isHorizontal ? box.width : box.height;
    } else {
      const itemSizeConst = isHorizontal ? filmStripItemSize.value : itemHeight.value;
      itemPos = displayIndex * itemSizeConst;
      itemSizeValue = itemSizeConst;
    }

    const itemCenter = itemPos + itemSizeValue / 2;
    const clientSize = isHorizontal ? el.clientWidth : el.clientHeight;
    
    // Calculate target scroll to center the item
    let targetScroll = itemCenter - clientSize / 2;
    
    // Clamp to bounds
    targetScroll = Math.max(0, targetScroll);
    const maxScroll = (isHorizontal ? el.scrollWidth : el.scrollHeight) - clientSize;
    targetScroll = Math.min(targetScroll, maxScroll);
    
    el.scrollTo({
      [isHorizontal ? 'left' : 'top']: targetScroll,
      behavior: center ? 'auto' : 'smooth'
    });
  }
}

function scrollToPosition(scrollTop: number) {
  if (scroller.value && !config.settings.grid.showFilmStrip) {
    scroller.value.$el.scrollTop = scrollTop;
  }
}

function scrollToRowIndex(rowIndex: number) {
  const box = layoutGeometry.value[rowIndex];
  if (box) {
    scrollToPosition(box.y);
    return;
  }
  if (!scroller.value || config.settings.grid.showFilmStrip) return;
  const nextScrollTop = Math.max(0, rowIndex) * itemHeight.value;
  scroller.value.$el.scrollTop = nextScrollTop;
}

function getColumnCount() {
  return columnCount.value;
}

function getScrollTop() {
  return scroller.value ? scroller.value.$el.scrollTop : 0;
}

function centerItem(index: number) {
  if (index >= 0) scrollToItem(index, true);
}

function getNextItemIndex(currentIndex: number, direction: 'up' | 'down'): number {
  const style = config.settings.grid.style;
  const supportsGeometryNavigation = style === 2 || (!config.settings.grid.showFilmStrip && isGeometryGridStyle(style));
  if (!supportsGeometryNavigation || layoutGeometry.value.length === 0) {
    return -1;
  }

  const currentDisplayIndex = fileIndexToRowIndex.value.get(currentIndex) ?? currentIndex;

  const currentBox = layoutGeometry.value[currentDisplayIndex];
  if (!currentBox) return currentIndex;

  const centerX = currentBox.x + currentBox.width / 2;
  const currentY = currentBox.y;
  
  // Find all items in the target direction
  let candidates: { index: number; box: Geometry; diffY: number }[] = [];

  layoutGeometry.value.forEach((box, displayIndex) => {
    if (direction === 'down') {
      if (box.y > currentY + 1) { // +1 for tolerance
         const fileIndex = hasGroupRows.value
           ? rowIndexToFileIndex.value.get(displayIndex)
           : displayIndex;
         if (fileIndex !== undefined) candidates.push({ index: fileIndex, box, diffY: box.y - currentY });
      }
    } else {
      if (box.y < currentY - 1) { // -1 for tolerance
         const fileIndex = hasGroupRows.value
           ? rowIndexToFileIndex.value.get(displayIndex)
           : displayIndex;
         if (fileIndex !== undefined) candidates.push({ index: fileIndex, box, diffY: currentY - box.y });
      }
    }
  });

  if (candidates.length === 0) return currentIndex;

  // Find the closest row (smallest diffY)
  const minDiffY = Math.min(...candidates.map(c => c.diffY));
  
  // Filter candidates to only those in the closest row
  const rowCandidates = candidates.filter(c => Math.abs(c.diffY - minDiffY) < 5); // 5px tolerance

  // Find item with closest centerX
  let closestIndex = -1;
  let minDistX = Infinity;

  rowCandidates.forEach(c => {
    const boxCenterX = c.box.x + c.box.width / 2;
    const dist = Math.abs(boxCenterX - centerX);
    if (dist < minDistX) {
      minDistX = dist;
      closestIndex = c.index;
    }
  });

  return closestIndex !== -1 ? closestIndex : currentIndex;
}

function getFileItem(item: any) {
  return isGroupRow(item) ? null : (item?.type === 'item' ? item.file : item);
}

function getFileIndex(item: any, displayIndex: number) {
  if (isGroupRow(item)) return -1;
  return Number(item?.file_index ?? displayIndex);
}

function getThumbnailKey(item: any, displayIndex: number) {
  const file = getFileItem(item);
  return `${getFileIndex(item, displayIndex)}:${file?.id || ''}`;
}

function isGroupRow(item: any) {
  return item?.type === 'group';
}

function getGroupSelectionState(item: any) {
  const groupId = String(item?.group_id || '');
  const selectedCount = Number(props.groupSelectedCounts[groupId] || 0);
  const count = Number(item?.count || 0);
  return {
    checked: count > 0 && selectedCount >= count,
    indeterminate: selectedCount > 0 && selectedCount < count,
  };
}

function isGroupSelectionLoading(item: any) {
  return Boolean(props.groupSelectionLoading[String(item?.group_id || '')]);
}

function getGroupIcon(item: any) {
  if (item?.icon) return item.icon;
  switch (Number(props.groupBy || 0)) {
    case GROUP.FOLDER:
      return IconFolder;
    case GROUP.DAY:
      return IconCalendarDay;
    case GROUP.MONTH:
    case GROUP.YEAR:
      return IconCalendarMonth;
    case GROUP.RATING:
      return IconStarFilled;
    case GROUP.LOCATION:
      return IconLocation;
    case GROUP.CAMERA:
      return IconCameraAperture;
    case GROUP.LENS:
      return IconCameraAperture;
    default:
      return IconFiles;
  }
}

function isFolderPathGroup() {
  return Number(props.groupBy || 0) === GROUP.FOLDER;
}

function getGroupTitleSegments(item: any) {
  const label = getFolderGroupLabel(item);
  if (!label) return [];
  if (label.includes(' > ')) {
    return label.split(' > ').map(part => part.trim()).filter(Boolean);
  }
  const normalized = label.replace(/\\/g, '/');
  return normalized.split('/').map(part => part.trim()).filter(Boolean);
}

function getFolderGroupBreadcrumbItems(item: any) {
  const folderPath = String(item?.label || '').trim();
  if (!folderPath || folderPath === 'Unknown folder') return [];

  const root = [...props.folderGroupRoots]
    .filter(root => root.path && isWithinRootPath(folderPath, root.path))
    .sort((a, b) => b.path.length - a.path.length)[0];

  if (root) {
    return buildFolderBreadcrumbs(folderPath, root.path, root.name || '');
  }

  return getGroupTitleSegments(item).map((label, index, segments) => ({
    label,
    path: segments.slice(0, index + 1).join('/'),
  }));
}

function getFolderGroupLabel(item: any) {
  const folderPath = String(item?.label || '').trim();
  if (!folderPath || folderPath === 'Unknown folder') return folderPath;

  const root = [...props.folderGroupRoots]
    .filter(root => root.path && isWithinRootPath(folderPath, root.path))
    .sort((a, b) => b.path.length - a.path.length)[0];
  if (!root) return folderPath;

  return formatFolderBreadcrumb(folderPath, root.path, root.name || '');
}

defineExpose({
  getColumnCount,
  scrollToPosition,
  scrollToRowIndex,
  getScrollTop,
  centerItem,
  refreshLayout: updateLayout,
  getNextItemIndex
});

</script>

<style scoped>
</style>

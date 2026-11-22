<template>
  <section class="w-full rounded-box bg-base-200 flex flex-col overflow-hidden shrink-0 border border-base-content/10">
    <header class="my-1 px-3 flex items-center gap-2 shrink-0">
      <span class="flex-1 text-xs font-normal text-primary/70">{{ $t('assign_tags.title') }}</span>
      <button type="button" class="cursor-pointer" :aria-label="$t('msgbox.close')" :disabled="busy" @mousedown.prevent @click.stop="$emit('close')"><IconClose class="size-4" /></button>
    </header>
    <div class="px-3 pb-0 pt-1 flex-1 min-h-0 flex flex-col gap-2">
      <div class="flex items-center gap-2 shrink-0">
        <div class="relative flex-1 min-w-0">
          <input ref="searchInput" v-model="search" class="input input-sm w-full shrink-0 pr-8" :placeholder="$t('menu.tag.search')"
            @focus="focusInput" @blur="blurInput" @keydown.stop="searchKey" />
          <button type="button" class="absolute right-2 top-1/2 -translate-y-1/2 shrink-0 cursor-pointer" :class="showSelectedOnly ? 'text-primary' : 'text-base-content/40 hover:text-base-content'"
            :title="$t('tag.show_selected_only')" :aria-label="$t('tag.show_selected_only')" :aria-pressed="showSelectedOnly"
            @click="showSelectedOnly = !showSelectedOnly"><IconFilter class="size-4" /></button>
        </div>
        <button type="button" class="text-xs shrink-0 cursor-pointer hover:text-primary" :class="recentOnly ? 'text-primary underline' : 'text-base-content/40'"
          :aria-pressed="recentOnly" @click="recentOnly = !recentOnly">{{ $t('assign_tags.recent_link') }}</button>
      </div>
      <p v-if="failed" role="alert" class="text-xs text-error">{{ $t('tag.load_failed') }}
        <button type="button" class="ml-2 underline" :disabled="busy" @click="load()">{{ $t('toolbar.tooltip.refresh') }}</button>
      </p>
      <p v-if="loading && !catalogReady" class="text-xs text-base-content/40">{{ $t('tooltip.loading') }}</p>
      <div ref="listContainer" class="min-h-0 relative flex-none" :style="{ height: listHeight + 'px' }" :aria-busy="loading || busy">
        <VirtualScroll ref="scroller" :items="rows" :item-size="ROW_HEIGHT" :geometry="geometry" :content-height="rows.length * ROW_HEIGHT">
        <template #default="{ item: tag }">
        <div :key="tag.id"
          role="treeitem" :aria-level="tag.depth + 1" :aria-expanded="tag.hasChildren ? !collapsed.includes(tag.id) || !!search : undefined"
          class="group h-full flex items-center text-sm py-1 pl-1 pr-1 rounded-box hover:bg-base-content/5 cursor-pointer select-none"
          :class="{ 'text-primary': selected.has(tag.id) }"
          @click="change(tag.id, !selected.has(tag.id))">
          <input type="checkbox" class="checkbox checkbox-xs mx-1 shrink-0" :class="{ 'checkbox-primary': selected.has(tag.id) || partial.has(tag.id) }" @click.stop :checked="selected.has(tag.id)" :indeterminate="partial.has(tag.id)"
            :disabled="loading || busy || failed || !fileIds.length" @change="change(tag.id, !selected.has(tag.id))" />
          <div class="flex items-center flex-1 min-w-0" :style="{ paddingLeft: `${tag.depth * 16}px` }">
            <button v-if="tag.hasChildren" type="button" class="size-5 shrink-0 flex items-center justify-center cursor-pointer" @click.stop="toggleBranch(tag.id)"><IconRight class="size-5 p-0.5" :class="{ 'rotate-90': search || !collapsed.includes(tag.id) }" /></button>
            <span v-else class="size-5 shrink-0" aria-hidden="true" />
            <span class="min-w-0 flex-1 truncate" :title="tag.path">{{ search && !showSelectedOnly ? tag.path : tag.name }}</span>
          </div>
        </div>
        </template>
        </VirtualScroll>
        <p v-if="!loading && !rows.length" class="absolute inset-0 flex items-center justify-center text-xs text-base-content/40 pointer-events-none">{{ $t('tag.not_found') }}</p>
      </div>
    </div>
    <div role="separator" aria-orientation="horizontal" tabindex="0"
      :aria-valuemin="MIN_LIST_HEIGHT" :aria-valuenow="listHeight"
      class="h-1 shrink-0 cursor-row-resize touch-none splitter-indicator splitter-horizontal"
      :class="{ 'splitter-dragging': isResizing }"
      @pointerdown.prevent="startResize" @keydown.up.prevent="setListHeight(listHeight - ROW_HEIGHT)" @keydown.down.prevent="setListHeight(listHeight + ROW_HEIGHT)">
    </div>
  </section>
</template>
<script setup lang="ts">
import { computed, ref, watch, nextTick, onBeforeUnmount, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { getAllTags, getTagSelectionCounts, applyTagsToFiles } from '@/common/api';
import { tagRows, ancestorIds, subtreeIds, type LibraryTag } from '@/common/tagTree';
import { config, libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import { IconClose, IconRight, IconFilter } from '@/common/icons';
import VirtualScroll from './VirtualScroll.vue';
const props = defineProps<{ fileIds: number[]; focusTagId?: number }>();
const emit = defineEmits(['close', 'changed']);
const ui = useUIStore();
const tags = ref<LibraryTag[]>([]), search = ref('');
const collapsed = computed<number[]>({
  get: () => libConfig.tag.collapsedTagIds || [],
  set: ids => { libConfig.tag.collapsedTagIds = ids; },
});
const selected = ref(new Set<number>()), partial = ref(new Set<number>());
function updateTagCount(tagId: number, count: number) {
  const all = new Set(selected.value), some = new Set(partial.value);
  all.delete(tagId); some.delete(tagId);
  if (count > 0 && count === props.fileIds.length) all.add(tagId);
  else if (count > 0) some.add(tagId);
  selected.value = all; partial.value = some;
}
const loading = ref(true), busy = ref(false), failed = ref(false);
const recent = ref<number[]>([]);
const recentOnly = ref(false), showSelectedOnly = ref(false);
const searchInput = ref<HTMLInputElement>();
const rows = computed(() => {
  const isAssigned = (id: number) => selected.value.has(id) || partial.value.has(id);
  if (recentOnly.value) {
    const query = search.value.trim().toLocaleLowerCase();
    return recent.value.slice(0, 10).map(id => tags.value.find(tag => tag.id === id))
      .filter((tag): tag is LibraryTag => !!tag && (!showSelectedOnly.value || isAssigned(tag.id)) && (!query || (tag.path || tag.name).toLocaleLowerCase().includes(query)))
      .map(tag => ({ ...tag, depth: 0, hasChildren: false }));
  }
  let catalog = tags.value;
  if (showSelectedOnly.value) {
    const included = new Set<number>();
    for (const tag of catalog) if (isAssigned(tag.id)) {
      included.add(tag.id);
      for (const id of ancestorIds(catalog, tag.id)) included.add(id);
    }
    catalog = catalog.filter(tag => included.has(tag.id));
  }
  return tagRows(catalog, search.value, showSelectedOnly.value ? [] : collapsed.value);
});
const ROW_HEIGHT = 32;
const MIN_LIST_HEIGHT = ROW_HEIGHT * 4;
const listHeight = computed(() => Math.max(MIN_LIST_HEIGHT, Number(config.infoPanel.assignTagsListHeight) || 240));
function setListHeight(height: number) {
  config.infoPanel.assignTagsListHeight = Math.max(MIN_LIST_HEIGHT, Math.round(height));
}
const isResizing = ref(false);
let resizeHandle: HTMLElement | null = null;
let resizePointer = -1, resizeY = 0, resizeHeight = 0;
function moveResize(event: PointerEvent) {
  if (event.pointerId === resizePointer) setListHeight(resizeHeight + event.clientY - resizeY);
}
function stopResize() {
  isResizing.value = false;
  const handle = resizeHandle;
  resizeHandle = null;
  if (handle) {
    handle.removeEventListener('pointermove', moveResize);
    handle.removeEventListener('pointerup', stopResize);
    handle.removeEventListener('pointercancel', stopResize);
    handle.removeEventListener('lostpointercapture', stopResize);
    if (handle.hasPointerCapture(resizePointer)) handle.releasePointerCapture(resizePointer);
  }
  resizePointer = -1;
}
function startResize(event: PointerEvent) {
  if (event.button !== 0) return;
  stopResize();
  resizeHandle = event.currentTarget as HTMLElement;
  resizePointer = event.pointerId; resizeY = event.clientY; resizeHeight = listHeight.value;
  resizeHandle.setPointerCapture(event.pointerId);
  isResizing.value = true;
  resizeHandle.addEventListener('pointermove', moveResize);
  resizeHandle.addEventListener('pointerup', stopResize);
  resizeHandle.addEventListener('pointercancel', stopResize);
  resizeHandle.addEventListener('lostpointercapture', stopResize);
}

const listContainer = ref<HTMLElement>();
const scroller = ref<InstanceType<typeof VirtualScroll>>();
const listWidth = ref(0);
const geometry = computed(() => rows.value.map((_, index) => ({ x: 0, y: index * ROW_HEIGHT, width: listWidth.value, height: ROW_HEIGHT })));
let listObserver: ResizeObserver | undefined;
watch([search, recentOnly, showSelectedOnly], () => scroller.value?.scrollToPosition(0));
watch(rows, async () => {
  await nextTick();
  const viewport = scroller.value?.$el as HTMLElement | undefined;
  if (viewport) scroller.value?.scrollToPosition(Math.min(viewport.scrollTop, Math.max(0, rows.value.length * ROW_HEIGHT - viewport.clientHeight)));
});
let version = 0, disposed = false;
let pendingFocusTagId = props.focusTagId || 0;
function focusSearch() { searchInput.value?.focus(); }
defineExpose({ locate, focusSearch });
const source = crypto.randomUUID();
let unlisten: (() => void) | undefined;
function focusInput() { if (!ui.isInputActive('AssignTagsPanel')) ui.pushInputHandler('AssignTagsPanel'); }
function blurInput() { ui.removeInputHandler('AssignTagsPanel'); }
function searchKey(event: KeyboardEvent) {
  if (event.key === 'Escape') { event.preventDefault(); searchInput.value?.blur(); }
  if (event.key === 'Enter') {
    event.preventDefault();
    const query = search.value.trim().toLocaleLowerCase();
    if (!query) return;
    const matches = rows.value.filter(tag => (tag.path || tag.name).toLocaleLowerCase().includes(query));
    const exact = matches.find(tag => tag.name.toLocaleLowerCase() === query || tag.path?.toLocaleLowerCase() === query);
    const target = exact || matches.find(tag => !selected.value.has(tag.id));
    if (target) void change(target.id, true);
  }
}
function toggleBranch(id: number) { if (search.value) return; collapsed.value = collapsed.value.includes(id) ? collapsed.value.filter(x => x !== id) : [...collapsed.value, id]; }
async function locate(tag: { id: number }) {
  search.value = ''; recentOnly.value = false; showSelectedOnly.value = false;
  const parents = new Set(ancestorIds(tags.value, tag.id));
  collapsed.value = collapsed.value.filter(id => !parents.has(id));
  await nextTick();
  const index = rows.value.findIndex(row => row.id === tag.id);
  const viewport = scroller.value?.$el as HTMLElement | undefined;
  if (index >= 0 && viewport) {
    const top = index * ROW_HEIGHT;
    if (top < viewport.scrollTop) scroller.value?.scrollToPosition(top);
    else if (top + ROW_HEIGHT > viewport.scrollTop + viewport.clientHeight)
      scroller.value?.scrollToPosition(top + ROW_HEIGHT - viewport.clientHeight);
  }
}
const catalogReady = ref(false);
let catalogPromise: Promise<any> | null = null;
let catalogSnapshot: any = null;
async function load(refreshCatalog = true) {
  const request = ++version, library = libConfig._libraryId, ids = [...props.fileIds];
  loading.value = true;
  failed.value = false;
  try {
    if (refreshCatalog || !catalogPromise) catalogPromise = getAllTags(0);
    const [catalog, counts] = await Promise.all([catalogPromise, ids.length ? getTagSelectionCounts(ids) : Promise.resolve([])]);
    if (disposed || request !== version || library !== libConfig._libraryId) return;
    if (!catalog || !counts) { if (!catalog) catalogPromise = null; failed.value = true; return; }
    if (catalog !== catalogSnapshot) {
      tags.value = catalog;
      catalogSnapshot = catalog;
    }
    catalogReady.value = true;
    selected.value = new Set(counts.filter((c: any) => Number(c.count) === ids.length).map((c: any) => Number(c.tag_id)));
    partial.value = new Set(counts.filter((c: any) => Number(c.count) > 0 && Number(c.count) < ids.length).map((c: any) => Number(c.tag_id)));
    if (pendingFocusTagId) { void locate({ id: pendingFocusTagId }); pendingFocusTagId = 0; }
  } finally { if (!disposed && request === version) loading.value = false; }
}
async function change(tagId: number, add: boolean) {
  if (busy.value || loading.value || failed.value || !props.fileIds.length) return;
  const library = libConfig._libraryId, ids = [...props.fileIds], selectionVersion = version;
  const tagIdInView = Number(libConfig.tag.id);
  const context = { source, library, tagId: tagIdInView, branch: subtreeIds(tags.value, tagIdInView), add };
  busy.value = true; failed.value = false;
  try {
    const result = await applyTagsToFiles(ids, add ? [tagId] : [], add ? [] : [tagId]);
    if (library !== libConfig._libraryId) return;
    if (result === null) { failed.value = true; return; }
    if (!disposed && selectionVersion === version) {
      updateTagCount(tagId, add ? ids.length : 0);
    }
    recent.value = [tagId, ...recent.value.filter(id => id !== tagId)].slice(0, 10);
    emit('changed', result, context);
  } finally {
    busy.value = false;
    // A concurrent refresh may have read membership before this write committed.
    // Reconcile only the original selection; ordinary saves stay entirely local.
    if (!disposed && library === libConfig._libraryId && selectionVersion !== version
      && JSON.stringify(ids) === JSON.stringify(props.fileIds)) await load(false);
  }
}
watch(() => libConfig._libraryId, () => {
  recent.value = [];
  catalogPromise = null; catalogSnapshot = null; catalogReady.value = false;
  tags.value = []; selected.value = new Set(); partial.value = new Set();
});
watch(() => JSON.stringify([libConfig._libraryId, ...props.fileIds]), () => {
  void load(false);
}, { immediate: true });
watch(() => props.focusTagId, id => {
  pendingFocusTagId = id || 0;
  if (id && !loading.value) { void locate({ id }); pendingFocusTagId = 0; }
});
onMounted(async () => {
  listObserver = new ResizeObserver(() => { listWidth.value = listContainer.value?.clientWidth || 0; });
  if (listContainer.value) { listWidth.value = listContainer.value.clientWidth; listObserver.observe(listContainer.value); }
  if (!props.focusTagId) { await nextTick(); if (!disposed) searchInput.value?.focus(); }
  const stop = await listen<{ source?: string }>('tags-changed', event => { if (event.payload?.source !== source) void load(true); });
  if (disposed) stop(); else unlisten = stop;
});
onBeforeUnmount(() => { disposed = true; stopResize(); version++; listObserver?.disconnect(); unlisten?.(); blurInput(); });
</script>

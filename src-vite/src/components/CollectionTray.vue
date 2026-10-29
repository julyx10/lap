<template>
  <section
    data-collection-tray-root="true"
    :data-collection-drop-new="collections.length === 0 ? 'true' : undefined"
    class="collection-tray min-h-0 flex flex-col  border-t border-base-content/5 shadow-sm"
    :class="libConfig.activePane === 'collection' ? '' : 'sidebar-pane-inactive'"
  >
    <div class="sidebar-panel-header cursor-pointer" @click="$emit('toggle-expanded')">
      <TButton
        :icon="IconRight"
        :buttonSize="'small'"
        iconClasses="transition-transform duration-200"
        :iconStyle="{ transform: expanded ? 'rotate(90deg)' : 'rotate(0deg)' }"
        @click.stop="$emit('toggle-expanded')"
      />
      <span class="sidebar-panel-header-title flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">
        {{ $t('collection.title') }}<template v-if="collections.length > 0"> ({{ collections.length.toLocaleString() }})</template>
      </span>
      <span v-if="isItemDragging" class="badge badge-sm badge-primary badge-outline shrink-0">
        {{ $t('collection.drop_title_hint') }}
      </span>
      <TButton
        v-if="expanded"
        :icon="IconAdd"
        :buttonSize="'small'"
        :tooltip="$t('collection.add')"
        :disabled="collections.length >= maxCollectionCount"
        @click.stop="addCollection"
      />
    </div>

    <transition
      enter-active-class="transition-all duration-200 ease-out"
      enter-from-class="opacity-0 -translate-y-1"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition-all duration-150 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-1"
    >
      <div v-if="expanded" class="min-h-0 flex-1 overflow-y-auto pb-1">
        <div v-if="collections.length > 10" class="mx-1 mb-2 px-1 shrink-0">
          <div
            :class="[
              'h-8 flex items-center rounded-box transition-colors bg-base-100/40',
              isSearchFocused ? 'border-2 border-primary' : 'border border-base-content/10 hover:border-base-content/30',
            ]"
          >
            <IconSearch class="ml-2 w-4 h-4 shrink-0" :class="isSearchFocused ? 'text-primary/70' : 'text-base-content/30'" />
            <input
              v-model="searchQuery"
              type="text"
              :placeholder="$t('collection.search')"
              class="w-full min-w-0 bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none"
              @focus="isSearchFocused = true"
              @blur="isSearchFocused = false"
            />
            <button
              v-if="searchQuery"
              type="button"
              class="mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70"
              @click="searchQuery = ''"
            >
              <IconClose class="w-4 h-4" />
            </button>
          </div>
        </div>
        <VueDraggable
          v-model="collections"
          class="px-1"
          :animation="200"
          handle=".collection-drag-handle"
          :disabled="Boolean(searchQuery) || renamingId !== null || isItemDragging || reorderingCollectionId === null"
          @start="onReorderStart"
          @end="onReorderEnd"
          @drop.stop
        >
          <div
            v-for="collection in filteredCollections"
            :key="collection.id"
            :data-reordering-collection="isReorderingCollection(collection) ? 'true' : undefined"
            :data-collection-drop-id="renamingId === collection.id ? undefined : collection.id"
            :class="[
              'sidebar-item group border-2 border-transparent',
              selectedId === collection.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="selectCollection(collection)"
          >
          <IconDragHandle
            v-if="isReorderingCollection(collection)"
            class="collection-drag-handle p-1 w-6 h-6 shrink-0 cursor-move text-base-content/70 hover:text-base-content"
            :title="$t('collection.reorder')"
          />
          <span v-else-if="reorderingCollectionId !== null" class="p-1 w-6 h-6 shrink-0"></span>
          <IconStack class="pr-1 w-6 h-6 shrink-0" />
          <input
            v-if="renamingId === collection.id"
            ref="renameInputRef"
            v-model="renameValue"
            class="input px-1 min-w-0 flex-1 text-base"
            maxlength="64"
            @click.stop
            @mousedown.stop
            @keydown.enter.prevent="commitRename(collection)"
            @keydown.escape.prevent="cancelRename"
            @blur="commitRename(collection)"
          />
          <span v-else class="sidebar-item-label">{{ collection.name }}</span>
          <div class="ml-auto flex flex-row items-center text-base-content/30">
            <span
              v-if="renamingId !== collection.id && getCollectionDisplayCount(collection) > 0"
              class="sidebar-item-count shrink-0"
            >
              {{ getCollectionDisplayCount(collection).toLocaleString() }}
            </span>
            <div
              v-if="renamingId !== collection.id"
              :class="[
                selectedId === collection.id ? '' : 'hidden group-hover:block',
              ]"
            >
              <ContextMenu
                :iconMenu="IconMore"
                :menuItems="collectionMenuItems(collection)"
                :smallIcon="true"
              />
            </div>
          </div>
          </div>
        </VueDraggable>
        <div v-if="collections.length > 0 && filteredCollections.length === 0" class="sidebar-empty text-sm">
          <span class="text-center">{{ $t('collection.not_found') }}</span>
        </div>
        <div
          v-if="!isLoadingCollections && collections.length === 0 && !renamingId"
          class="mt-2 px-3 py-3 flex flex-col items-center gap-1 text-center text-base-content/30"
        >
          <span class="text-sm">{{ $t('collection.empty_content') }}</span>
          <span class="text-xs">{{ $t('collection.drop_here') }}</span>
        </div>
      </div>
    </transition>

    <MessageBox
      v-if="deleteTarget"
      :title="$t('collection.delete_confirm_title')"
      :message="$t('collection.delete_confirm_message', { name: deleteTarget.name })"
      :OkText="$t('collection.delete_confirm_ok')"
      :cancelText="$t('msgbox.cancel')"
      :warningOk="true"
      @ok="confirmDelete"
      @cancel="deleteTarget = null"
    />
    <MessageBox
      v-if="clearTarget"
      :title="$t('collection.clear_confirm_title')"
      :message="$t('collection.clear_confirm_message', { name: clearTarget.name })"
      :OkText="$t('collection.clear_confirm_ok')"
      :cancelText="$t('msgbox.cancel')"
      @ok="confirmClear"
      @cancel="clearTarget = null"
    />
  </section>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { emit as tauriEmit, listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config, libConfig } from '@/common/config';
import { clearCollection, createCollection, deleteCollection as deleteCollectionApi, getCollectionCounts, listCollections, renameCollection, reorderCollections } from '@/common/api';
import { IconAdd, IconRight, IconRename, IconMore, IconStack, IconRemove, IconTrash, IconClose, IconSearch, IconDragHandle, IconOrder } from '@/common/icons';
import { VueDraggable } from 'vue-draggable-plus';
import ContextMenu from '@/components/ContextMenu.vue';
import MessageBox from '@/components/MessageBox.vue';
import TButton from '@/components/TButton.vue';

defineProps({
  expanded: {
    type: Boolean,
    required: true,
  },
});

const emit = defineEmits(['toggle-expanded']);

const { t } = useI18n();
const uiStore = useUIStore();
type Collection = {
  id: number;
  name: string;
  sortOrder?: number;
  count: number;
};

const collections = ref<Collection[]>([]);
const isLoadingCollections = ref(true);
const maxCollectionCount = computed(() => Math.max(1, Number(config.main.maxCollectionCount) || 100));
const searchQuery = ref('');
const isSearchFocused = ref(false);
const filteredCollections = computed(() => {
  const query = searchQuery.value.trim().toLocaleLowerCase();
  return query
    ? collections.value.filter(collection => collection.name.toLocaleLowerCase().includes(query))
    : collections.value;
});
watch(() => collections.value.length, (count) => {
  if (count <= 10) searchQuery.value = '';
});
watch(() => libConfig.activePane, () => {
  if (libConfig.activePane === 'collection') void loadCollections();
});
watch(() => config.settings.groupRawJpegPairs, () => { void loadCollections(); });
const selectedId = ref<number | null>(Number(libConfig.collection.selectedId || 0) || null);
const getCollectionDisplayCount = (collection: Collection) => Number(libConfig.collection.counts?.[String(collection.id)] || 0);
const reorderingCollectionId = ref<number | null>(null);
const renamingId = ref<number | null>(null);
const renameValue = ref('');
const renameInputRef = ref<HTMLInputElement | HTMLInputElement[] | null>(null);
const isItemDragging = ref(false);
const deleteTarget = ref<Collection | null>(null);
const clearTarget = ref<Collection | null>(null);
let unlistenCollectionFilesDropped: (() => void) | null = null;
let unlistenCollectionsChanged: (() => void) | null = null;
let unlistenContentItemsDragState: (() => void) | null = null;
let unlistenLibrarySwitched: (() => void) | null = null;
let collectionCountRequest = 0;
let isCollectionTrayMounted = true;

onMounted(async () => {
  document.addEventListener('pointerdown', handleReorderOutsidePointerDown, true);
  try {
    await loadCollections();
  } finally {
    isLoadingCollections.value = false;
  }
  unlistenCollectionFilesDropped = await listen('collection-files-dropped', async () => {
    await loadCollections();
  });
  unlistenCollectionsChanged = await listen('collections-changed', async () => {
    await loadCollections();
  });
  unlistenContentItemsDragState = await listen('content-items-drag-state', (event: any) => {
    isItemDragging.value = Boolean(event.payload?.dragging);
  });
  unlistenLibrarySwitched = await listen('library-switched', async () => {
    selectedId.value = Number(libConfig.collection.selectedId || 0) || null;
    reorderingCollectionId.value = null;
    renamingId.value = null;
    renameValue.value = '';
    deleteTarget.value = null;
    await loadCollections();
  });
});

onBeforeUnmount(() => {
  isCollectionTrayMounted = false;
  collectionCountRequest++;
  document.removeEventListener('pointerdown', handleReorderOutsidePointerDown, true);
  uiStore.removeInputHandler('CollectionTrayDrag');
  unlistenCollectionFilesDropped?.();
  unlistenCollectionFilesDropped = null;
  unlistenCollectionsChanged?.();
  unlistenCollectionsChanged = null;
  unlistenContentItemsDragState?.();
  unlistenContentItemsDragState = null;
  unlistenLibrarySwitched?.();
  unlistenLibrarySwitched = null;
});

async function loadCollections(preferredId?: number) {
  const request = ++collectionCountRequest;
  const libraryId = libConfig._libraryId;
  const [result, counts] = await Promise.all([
    listCollections(),
    getCollectionCounts(),
  ]);
  if (
    !isCollectionTrayMounted
    || request !== collectionCountRequest
    || libraryId !== libConfig._libraryId
  ) return;
  const countMap = counts || {};
  libConfig.collection.counts = countMap;
  collections.value = Array.isArray(result)
    ? result.map((item: any) => ({
      id: Number(item.id),
      name: String(item.name || ''),
      sortOrder: Number(item.sortOrder || 0),
      count: Number(countMap[String(item.id)] || 0),
    }))
    : [];

  const nextSelectedId = Number(preferredId || libConfig.collection.selectedId || selectedId.value || 0);
  const selected = collections.value.find(item => item.id === nextSelectedId) || null;
  if (selected) {
    selectedId.value = selected.id;
    libConfig.collection.selectedId = selected.id;
  } else {
    selectedId.value = null;
    libConfig.collection.selectedId = null;
  }
}

function selectCollection(collection: Collection) {
  libConfig.activePane = 'collection';
  selectedId.value = collection.id;
  libConfig.collection.selectedId = collection.id;
  uiStore.requestCountUpdate({ source: 'collection', id: Number(collection.id) });
  libConfig.collection.activateTick = Number(libConfig.collection.activateTick || 0) + 1;
}

async function addCollection() {
  if (collections.value.length >= maxCollectionCount.value) return;
  const collection = await createCollection(t('collection.default_name', { index: collections.value.length + 1 }));
  if (!collection?.id) return;
  await loadCollections(Number(collection.id));
  const created = collections.value.find(item => item.id === Number(collection.id));
  if (created) {
    selectCollection(created);
    startRename(created);
  }
}

async function startRename(collection: Collection) {
  selectCollection(collection);
  renamingId.value = collection.id;
  renameValue.value = collection.name;
  await nextTick();
  const input = Array.isArray(renameInputRef.value)
    ? renameInputRef.value[0]
    : renameInputRef.value;
  input?.focus({ preventScroll: true });
  input?.select();
}

async function commitRename(collection: Collection) {
  if (renamingId.value !== collection.id) return;
  const nextName = renameValue.value.trim();
  if (nextName && nextName !== collection.name) {
    await renameCollection(collection.id, nextName);
    await loadCollections(collection.id);
    await tauriEmit('refresh-content');
  }
  cancelRename();
}

function cancelRename() {
  renamingId.value = null;
  renameValue.value = '';
}

function deleteCollection(collection: Collection) {
  deleteTarget.value = collection;
}

async function confirmDelete() {
  const target = deleteTarget.value;
  if (!target) return;
  const wasSelected = Number(libConfig.collection.selectedId || 0) === target.id;
  const targetIndex = collections.value.findIndex(item => item.id === target.id);
  const replacementId = wasSelected && targetIndex >= 0
    ? collections.value[targetIndex + 1]?.id || collections.value[targetIndex - 1]?.id
    : undefined;
  deleteTarget.value = null;
  await deleteCollectionApi(target.id);
  await loadCollections(replacementId);
  await tauriEmit('refresh-content');
}

async function confirmClear() {
  const target = clearTarget.value;
  if (!target) return;
  clearTarget.value = null;
  await clearCollection(target.id);
  await loadCollections(Number(libConfig.collection.selectedId || 0));
  await tauriEmit('refresh-content');
}

function collectionMenuItems(collection: Collection) {
  return [
    {
      label: t('collection.rename'),
      icon: IconRename,
      action: () => startRename(collection),
    },
    {
      label: t('collection.reorder'),
      icon: IconOrder,
      action: () => {
        searchQuery.value = '';
        reorderingCollectionId.value = reorderingCollectionId.value === collection.id ? null : collection.id;
      },
    },
    {
      label: t('collection.clear'),
      action: () => { clearTarget.value = collection; },
    },
    { label: "-", action: null },
    {
      label: t('collection.delete'),
      icon: IconTrash,
      action: () => deleteCollection(collection),
    },
  ];
}

const isReorderingCollection = (collection: Collection) => reorderingCollectionId.value === collection.id;

function handleReorderOutsidePointerDown(event: PointerEvent) {
  if (event.button !== 0 || reorderingCollectionId.value === null) return;
  if (event.target instanceof Element && event.target.closest('[data-reordering-collection="true"]')) return;
  reorderingCollectionId.value = null;
}

function onReorderStart() {
  uiStore.removeInputHandler('CollectionTrayDrag');
  uiStore.pushInputHandler('CollectionTrayDrag');
}

async function onReorderEnd() {
  setTimeout(() => uiStore.removeInputHandler('CollectionTrayDrag'), 0);
  try {
    await reorderCollections(collections.value.map((collection, sortOrder) => ({ id: collection.id, sortOrder })));
  } catch (error) {
    console.error('Failed to reorder collections:', error);
    await loadCollections();
  }
}
</script>

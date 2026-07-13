<template>
  <section
    data-collection-tray-root="true"
    :data-collection-drop-new="collections.length === 0 ? 'true' : undefined"
    class="collection-tray min-h-0 flex flex-col rounded-box bg-base-300/70 border border-base-content/5 shadow-sm"
    :class="libConfig.activePane === 'collection' ? '' : 'sidebar-pane-inactive'"
  >
    <div class="sidebar-panel-header cursor-pointer" @click="$emit('toggle-expanded')">
      <span class="sidebar-panel-header-title flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">
        {{ $t('collection.title') }}
      </span>
      <span v-if="isItemDragging" class="badge badge-sm badge-primary badge-outline shrink-0">
        {{ $t('collection.drop_title_hint') }}
      </span>
      <TButton
        v-if="expanded"
        :icon="IconAdd"
        :buttonSize="'small'"
        :tooltip="$t('collection.add')"
        :disabled="collections.length >= MAX_COLLECTIONS"
        @click.stop="addCollection"
      />
      <TButton
        :icon="expanded ? IconArrowDown : IconArrowUp"
        :buttonSize="'small'"
        @click.stop="$emit('toggle-expanded')"
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
        <div
          v-for="collection in collections"
          :key="collection.id"
          :data-collection-drop-id="renamingId === collection.id ? undefined : collection.id"
          :class="[
            'sidebar-item group border-2 border-transparent',
            selectedId === collection.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
          ]"
          @click="selectCollection(collection)"
        >
          <IconBookmark class="mx-1 w-5 h-5 shrink-0" />
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
          <span
            v-if="renamingId !== collection.id"
            :class="[
              'sidebar-item-count ml-auto',
              selectedId === collection.id ? 'hidden' : 'group-hover:hidden',
            ]"
          >
            {{ collection.count.toLocaleString() }}
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
        <div
          v-if="collections.length === 0 && !renamingId"
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
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { emit as tauriEmit, listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { libConfig } from '@/common/config';
import { clearCollection, createCollection, deleteCollection as deleteCollectionApi, listCollections, renameCollection } from '@/common/api';
import { IconAdd, IconArrowDown, IconArrowUp, IconEdit, IconMore, IconBookmark, IconRemove, IconTrash } from '@/common/icons';
import ContextMenu from '@/components/ContextMenu.vue';
import MessageBox from '@/components/MessageBox.vue';
import TButton from '@/components/TButton.vue';

const props = defineProps({
  expanded: {
    type: Boolean,
    required: true,
  },
});

const emit = defineEmits(['toggle-expanded']);

const { t } = useI18n();
const MAX_COLLECTIONS = 10;

type Collection = {
  id: number;
  name: string;
  count: number;
  sortOrder?: number;
};

const collections = ref<Collection[]>([]);
const selectedId = ref<number | null>(Number(libConfig.collection.selectedId || 0) || null);
const renamingId = ref<number | null>(null);
const renameValue = ref('');
const renameInputRef = ref<HTMLInputElement | HTMLInputElement[] | null>(null);
const isItemDragging = ref(false);
const deleteTarget = ref<Collection | null>(null);
const clearTarget = ref<Collection | null>(null);
let unlistenCollectionFilesDropped: (() => void) | null = null;
let unlistenContentItemsDragState: (() => void) | null = null;
let unlistenLibrarySwitched: (() => void) | null = null;

onMounted(async () => {
  await loadCollections();
  unlistenCollectionFilesDropped = await listen('collection-files-dropped', async () => {
    await loadCollections();
  });
  unlistenContentItemsDragState = await listen('content-items-drag-state', (event: any) => {
    isItemDragging.value = Boolean(event.payload?.dragging);
  });
  unlistenLibrarySwitched = await listen('library-switched', async () => {
    selectedId.value = Number(libConfig.collection.selectedId || 0) || null;
    renamingId.value = null;
    renameValue.value = '';
    deleteTarget.value = null;
    await loadCollections();
  });
});

watch(
  () => libConfig.activePane,
  (pane) => {
    if (pane === 'main' && props.expanded) {
      emit('toggle-expanded');
    }
  },
);

onBeforeUnmount(() => {
  unlistenCollectionFilesDropped?.();
  unlistenCollectionFilesDropped = null;
  unlistenContentItemsDragState?.();
  unlistenContentItemsDragState = null;
  unlistenLibrarySwitched?.();
  unlistenLibrarySwitched = null;
});

async function loadCollections(preferredId?: number) {
  const result = await listCollections();
  collections.value = Array.isArray(result)
    ? result.map((item: any) => ({
      id: Number(item.id),
      name: String(item.name || ''),
      count: Number(item.count || 0),
      sortOrder: Number(item.sortOrder || 0),
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
}

async function addCollection() {
  if (collections.value.length >= MAX_COLLECTIONS) return;
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
  deleteTarget.value = null;
  await deleteCollectionApi(target.id);
  await loadCollections();
  if (wasSelected) {
    const first = collections.value[0] || null;
    if (first) selectCollection(first);
    else {
      selectedId.value = null;
      libConfig.collection.selectedId = null;
    }
  }
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
      icon: IconEdit,
      action: () => startRename(collection),
    },
    {
      label: t('collection.clear'),
      disabled: collection.count === 0,
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
</script>

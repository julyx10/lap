<template>
  <section
    data-collection-tray-root="true"
    class="collection-tray min-h-0 flex flex-col"
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
        :icon="IconAdd"
        :buttonSize="'small'"
        :tooltip="$t('collection.add')"
        @click.stop="addCollection"
      />
      <TButton
        :icon="expanded ? IconArrowDown : IconArrowUp"
        :buttonSize="'small'"
        :tooltip="expanded ? $t('collection.collapse') : $t('collection.expand')"
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
            maxlength="80"
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
          v-if="allCollectionsEmpty && !renamingId"
          class="mt-2 text-xs text-center text-base-content/30"
        >
          {{ $t('collection.empty_drop_hint') }}
        </div>
      </div>
    </transition>
  </section>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { libConfig } from '@/common/config';
import { IconAdd, IconArrowDown, IconArrowUp, IconEdit, IconMore, IconBookmark, IconTrash } from '@/common/icons';
import ContextMenu from '@/components/ContextMenu.vue';
import TButton from '@/components/TButton.vue';

defineProps({
  expanded: {
    type: Boolean,
    required: true,
  },
});

defineEmits(['toggle-expanded']);

const { t } = useI18n();

type Collection = {
  id: string;
  name: string;
  count: number;
};

const collections = ref<Collection[]>([
  { id: 'default', name: 'Collection 1', count: 0 },
]);
const selectedId = ref(libConfig.collection.selectedId || collections.value[0].id);
const renamingId = ref<string | null>(null);
const renameValue = ref('');
const renameInputRef = ref<HTMLInputElement | HTMLInputElement[] | null>(null);
const isItemDragging = ref(false);
const allCollectionsEmpty = computed(() => collections.value.every(collection => collection.count === 0));
let unlistenCollectionFilesDropped: (() => void) | null = null;
let unlistenContentItemsDragState: (() => void) | null = null;

onMounted(async () => {
  unlistenCollectionFilesDropped = await listen('collection-files-dropped', (event: any) => {
    const collectionId = String(event.payload?.collectionId || '');
    const count = Number(event.payload?.count || 0);
    if (!collectionId || count <= 0) return;
    const collection = collections.value.find(item => item.id === collectionId);
    if (!collection) return;
    collection.count += count;
  });
  unlistenContentItemsDragState = await listen('content-items-drag-state', (event: any) => {
    isItemDragging.value = Boolean(event.payload?.dragging);
  });
});

onBeforeUnmount(() => {
  unlistenCollectionFilesDropped?.();
  unlistenCollectionFilesDropped = null;
  unlistenContentItemsDragState?.();
  unlistenContentItemsDragState = null;
});

function selectCollection(collection: Collection) {
  libConfig.activePane = 'collection';
  selectedId.value = collection.id;
  libConfig.collection.selectedId = collection.id;
}

async function addCollection() {
  const collection: Collection = {
    id: crypto.randomUUID?.() || `${Date.now()}`,
    name: `Collection ${collections.value.length + 1}`,
    count: 0,
  };
  collections.value = [...collections.value, collection];
  selectCollection(collection);
  startRename(collection);
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

function commitRename(collection: Collection) {
  if (renamingId.value !== collection.id) return;
  const nextName = renameValue.value.trim();
  if (nextName) {
    collection.name = nextName;
  }
  cancelRename();
}

function cancelRename() {
  renamingId.value = null;
  renameValue.value = '';
}

function deleteCollection(collection: Collection) {
  if (collections.value.length <= 1) return;
  collections.value = collections.value.filter(item => item.id !== collection.id);
  if (selectedId.value === collection.id) {
    selectCollection(collections.value[0]);
  }
}

function collectionMenuItems(collection: Collection) {
  return [
    {
      label: t('collection.rename'),
      icon: IconEdit,
      action: () => startRename(collection),
    },
    {
      label: t('collection.delete'),
      icon: IconTrash,
      disabled: collections.value.length <= 1,
      action: () => deleteCollection(collection),
    },
  ];
}
</script>

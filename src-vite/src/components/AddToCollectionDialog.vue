<template>
  <ModalDialog :title="$t('collection.edit_collections') + (collections.length ? ` (${collections.length.toLocaleString()})` : '')" :width="500" :position-key="positionKey" @cancel="close">
    <template #title-actions>
      <button
        type="button"
        class="t-button-default btn-outline gap-1.5"
        :disabled="creatingCollection || applying || isAddingCollection || renamingId !== null || collections.length >= config.main.maxCollectionCount"
        @click="startNewCollection"
      >
        <IconAdd class="w-4 h-4" aria-hidden="true" />
        {{ $t('collection.add') }}
      </button>
    </template>
    <section class="space-y-3">
      <div class="flex items-center gap-2">
        <div class="grow h-8 flex items-center rounded-box overflow-hidden bg-base-100 border border-neutral-content/30 focus-within:border-primary">
          <IconSearch class="ml-2 w-4 h-4 text-base-content/70" />
          <input ref="searchInput" v-model="query" :placeholder="$t('collection.search')" class="w-full bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none" />
          <button v-if="query" class="mr-1 p-1 text-base-content/30 hover:text-base-content/70" @click="query = ''">
            <IconClose class="w-4 h-4" />
          </button>
        </div>
      </div>
      <div class="min-h-48 max-h-[50vh] overflow-y-auto rounded-box p-2 bg-base-100/30 border border-base-content/10 select-none">
        <div v-if="isAddingCollection" class="px-7 py-1 flex items-center gap-1 text-primary">
          <IconCollection class="w-4 h-4 shrink-0" />
          <input
            ref="newCollectionNameInput"
            v-model="newCollectionName"
            type="text"
            maxlength="64"
            class="input input-sm px-2 min-w-0 flex-1 text-sm"
            :aria-label="$t('collection.add')"
            :readonly="creatingCollection"
            @click.stop
            @keydown.stop
            @keydown.enter.prevent="!$event.isComposing && addNewCollection()"
            @keydown.escape.prevent="cancelNewCollection"
            @blur="onNewCollectionBlur"
            @input="newCollectionNameEdited = true"
          />
        </div>
        <div
          v-for="collection in visibleCollections"
          :key="collection.id"
          :ref="element => setCollectionRowRef(Number(collection.id), element)"
          class="group w-full p-2 flex items-center gap-1 rounded-box text-left cursor-pointer hover:bg-base-content/5 transition-colors"
          :class="isSelected(collection.id) ? 'text-primary' : ''"
          tabindex="0"
          @click="toggle(collection.id)"
          @keydown.enter.prevent="toggle(collection.id)"
          @keydown.space.prevent="toggle(collection.id)"
        >
          <label class="flex items-center cursor-pointer shrink-0" @click.stop @dblclick.stop>
            <input
              type="checkbox"
              class="checkbox checkbox-xs"
              :class="isSelected(collection.id) ? 'checkbox-primary opacity-70' : ''"
              :checked="isSelected(collection.id)"
              :indeterminate="isIntermediate(collection.id)"
              :disabled="!hasSelectedFiles"
              @change="toggle(collection.id)"
            />
          </label>
          <IconCollection class="w-4 h-4 shrink-0" />
          <input
            v-if="renamingId === Number(collection.id)"
            ref="renameInput"
            v-model="renameValue"
            class="input px-1 min-w-0 flex-1 text-sm"
            maxlength="64"
            @click.stop
            @mousedown.stop
            @keydown.stop
            @keydown.enter.prevent="commitRename(collection)"
            @keydown.escape.prevent="cancelRename"
            @blur="commitRename(collection)"
          />
          <span v-else class="min-w-0 flex-1 truncate">{{ collection.name }}</span>
          <span
            v-if="renamingId !== Number(collection.id) && Number(collection.count || 0) > 0"
            class="sidebar-item-count shrink-0 group-hover:hidden"
          >
            {{ Number(collection.count || 0).toLocaleString() }}
          </span>
          <div
            v-if="renamingId !== Number(collection.id)"
            class="shrink-0 hidden group-hover:flex"
          >
            <button type="button" class="p-1 text-base-content/40 hover:text-base-content cursor-pointer" :title="$t('collection.rename')" @click.stop="startRename(collection)">
              <IconRename class="w-4 h-4" />
            </button>
            <button type="button" class="p-1 text-base-content/40 hover:text-error cursor-pointer" :title="$t('collection.delete')" @click.stop="deleteCollection(collection)">
              <IconTrash class="w-4 h-4 cursor-pointer" />
            </button>
          </div>
        </div>
        <div v-if="visibleCollections.length === 0 && !isAddingCollection" class="min-h-24 flex items-center justify-center text-sm text-base-content/30">
          {{ $t('collection.not_found') }}
        </div>
      </div>
    </section>
    <div class="mt-4 flex justify-end gap-3">
      <button class="t-button-default" @click="close">{{ $t('msgbox.cancel') }}</button>
      <button class="t-button-primary" :disabled="changes.size === 0 || applying || creatingCollection || isAddingCollection" @click="apply">{{ $t('msgbox.ok') }}</button>
    </div>
  </ModalDialog>
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
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import { emit as tauriEmit } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useToast } from '@/common/toast';
import { addFilesToCollection, createCollection, deleteCollection as deleteCollectionApi, getCollectionSelectionCounts, listCollections, removeFilesFromCollection, renameCollection } from '@/common/api';
import { IconAdd, IconCollection, IconClose, IconRename, IconSearch, IconTrash } from '@/common/icons';
import { config, libConfig } from '@/common/config';
import MessageBox from '@/components/MessageBox.vue';
import ModalDialog from '@/components/ModalDialog.vue';
import { useUIStore } from '@/stores/uiStore';

const props = defineProps<{ fileIds: number[]; positionKey?: string }>();
const emit = defineEmits(['applied', 'cancel', 'deleted']);
const collections = ref<any[]>([]);
const query = ref('');
const selectionCounts = ref(new Map<number, number>());
const changes = ref(new Map<number, 'add' | 'remove'>());
const applying = ref(false);
const creatingCollection = ref(false);
const isAddingCollection = ref(false);
const searchInput = ref<HTMLInputElement | null>(null);
const newCollectionNameInput = ref<HTMLInputElement | null>(null);
const newCollectionName = ref('');
const newCollectionNameEdited = ref(false);
const renamingId = ref<number | null>(null);
const renameValue = ref('');
const renameInput = ref<HTMLInputElement | HTMLInputElement[] | null>(null);
const deleteTarget = ref<any | null>(null);
const collectionRowRefs = new Map<number, Element>();
const uiStore = useUIStore();
const toast = useToast();
const { t } = useI18n();

function showNameSaveError(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  toast.error(t(message.includes('already exists') ? 'collection.name_exists' : 'collection.name_save_failed'));
}

const matchingCollections = computed(() => {
  const normalized = query.value.trim().toLocaleLowerCase();
  return normalized ? collections.value.filter(c => String(c.name).toLocaleLowerCase().includes(normalized)) : collections.value;
});
const visibleCollections = computed(() => matchingCollections.value);
const hasSelectedFiles = computed(() => props.fileIds.some(id => Number(id) > 0));

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('AddToCollectionDialog');
  await loadCollections();
  await loadSelectionCounts();
  await nextTick();
  searchInput.value?.focus();
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('AddToCollectionDialog');
});

function close() {
  if (!applying.value) emit('cancel');
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape' && uiStore.isInputActive('AddToCollectionDialog')) {
    event.preventDefault();
    close();
  }
}

async function toggle(id: number) {
  if (!hasSelectedFiles.value) return;
  const normalized = Number(id);
  const next = new Map(changes.value);
  const current = next.get(normalized);
  const initialCount = selectionCounts.value.get(normalized) || 0;
  if (current === 'add') {
    initialCount === 0 ? next.delete(normalized) : next.set(normalized, 'remove');
  } else if (current === 'remove') {
    next.delete(normalized);
  }
  else next.set(normalized, isSelected(normalized) ? 'remove' : 'add');
  changes.value = next;
  await nextTick();
  collectionRowRefs.get(normalized)?.scrollIntoView({ block: 'nearest' });
}

function isSelected(id: number) {
  if (!hasSelectedFiles.value) return false;
  const change = changes.value.get(Number(id));
  return change === 'add' || (change === undefined && selectionCounts.value.get(Number(id)) === props.fileIds.length);
}

function isIntermediate(id: number) {
  if (!hasSelectedFiles.value) return false;
  if (changes.value.has(Number(id))) return false;
  const count = selectionCounts.value.get(Number(id)) || 0;
  return count > 0 && count < props.fileIds.length;
}

function setCollectionRowRef(id: number, element: Element | null) {
  if (element) collectionRowRefs.set(id, element);
  else collectionRowRefs.delete(id);
}

async function loadCollections() {
  const result = await listCollections();
  const sidebarCounts = libConfig.collection.counts || {};
  collections.value = (result || []).map((collection: any) => ({
    ...collection,
    count: Number(sidebarCounts[String(collection.id)] || 0),
  }));
}

async function loadSelectionCounts() {
  if (!props.fileIds.length) return;
  const counts = await getCollectionSelectionCounts(props.fileIds);
  if (counts === null) return;
  selectionCounts.value = new Map(counts.map((entry: any) => [Number(entry.collection_id), Number(entry.count)]));
}

async function startNewCollection() {
  if (creatingCollection.value || applying.value || collections.value.length >= config.main.maxCollectionCount) return;
  if (isAddingCollection.value) {
    newCollectionNameInput.value?.focus();
    return;
  }
  cancelRename();
  query.value = '';
  const names = new Set(collections.value.map(collection => String(collection.name).toLocaleLowerCase()));
  const baseName = t('collection.add');
  let name = baseName;
  let index = 0;
  while (names.has(name.toLocaleLowerCase())) name = `${baseName} ${++index}`;
  newCollectionName.value = name;
  newCollectionNameEdited.value = false;
  isAddingCollection.value = true;
  await nextTick();
  newCollectionNameInput.value?.focus();
  newCollectionNameInput.value?.select();
}

function cancelNewCollection() {
  if (creatingCollection.value) return;
  isAddingCollection.value = false;
  newCollectionName.value = '';
  newCollectionNameEdited.value = false;
}

function onNewCollectionBlur() {
  if (!newCollectionNameEdited.value) cancelNewCollection();
  else addNewCollection();
}

async function addNewCollection() {
  const name = newCollectionName.value.trim();
  if (!isAddingCollection.value || creatingCollection.value || applying.value || collections.value.length >= config.main.maxCollectionCount) return;
  if (!name) {
    cancelNewCollection();
    return;
  }
  creatingCollection.value = true;
  try {
    const collection = await createCollection(name);
    if (!collection?.id) {
      toast.error(t('collection.name_save_failed'));
      return;
    }
    await loadCollections();
    if (hasSelectedFiles.value) {
      const next = new Map(changes.value);
      next.set(Number(collection.id), 'add');
      changes.value = next;
    }
    isAddingCollection.value = false;
    newCollectionName.value = '';
    query.value = '';
    await nextTick();
    collectionRowRefs.get(Number(collection.id))?.scrollIntoView({ block: 'nearest' });
    await tauriEmit('collections-changed');
  } catch (error) {
    showNameSaveError(error);
  } finally {
    creatingCollection.value = false;
  }
}

async function startRename(collection: any) {
  renamingId.value = Number(collection.id);
  renameValue.value = String(collection.name || '');
  await nextTick();
  const input = Array.isArray(renameInput.value) ? renameInput.value[0] : renameInput.value;
  input?.focus({ preventScroll: true });
  input?.select();
}

function cancelRename() {
  renamingId.value = null;
  renameValue.value = '';
}

async function commitRename(collection: any) {
  if (renamingId.value !== Number(collection.id)) return;
  const name = renameValue.value.trim();
  if (name && name !== collection.name) {
    try {
      await renameCollection(Number(collection.id), name);
      await loadCollections();
      await tauriEmit('collections-changed');
    } catch (error) {
      showNameSaveError(error);
    }
  }
  cancelRename();
}

function deleteCollection(collection: any) {
  deleteTarget.value = collection;
}

async function confirmDelete() {
  const target = deleteTarget.value;
  if (!target) return;
  deleteTarget.value = null;
  await deleteCollectionApi(Number(target.id));
  const next = new Map(changes.value);
  next.delete(Number(target.id));
  changes.value = next;
  selectionCounts.value.delete(Number(target.id));
  await loadCollections();
  await tauriEmit('collections-changed');
  emit('deleted', Number(target.id));
}

async function apply() {
  const fileIds = [...new Set(props.fileIds.map(Number).filter(id => id > 0))];
  if (!fileIds.length || !changes.value.size || applying.value || creatingCollection.value || isAddingCollection.value) return;
  applying.value = true;
  try {
    const pendingChanges = [...changes.value];
    const settled: PromiseSettledResult<any>[] = [];
    for (const [collectionId, action] of pendingChanges) {
      try {
        settled.push({
          status: 'fulfilled',
          value: action === 'add'
            ? await addFilesToCollection(collectionId, fileIds)
            : await removeFilesFromCollection(collectionId, fileIds),
        });
      } catch (reason) {
        settled.push({ status: 'rejected', reason });
      }
    }
    const successfulChanges = pendingChanges.filter((_, index) => settled[index].status === 'fulfilled');
    const results = settled.flatMap((result, index) =>
      result.status === 'fulfilled' && pendingChanges[index][1] === 'add' ? [result.value] : []
    );
    const changedCollectionIds = successfulChanges.map(([collectionId]) => collectionId);
    const removedCollectionIds = successfulChanges
      .filter(([, action]) => action === 'remove')
      .map(([collectionId]) => collectionId);
    await tauriEmit('collections-changed');
    emit('applied', { fileIds, results, changedCollectionIds, removedCollectionIds, failed: settled.length - successfulChanges.length });
  } finally {
    applying.value = false;
  }
}
</script>

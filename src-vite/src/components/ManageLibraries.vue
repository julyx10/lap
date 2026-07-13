<template>
  <ModalDialog 
    :title="$t('msgbox.manage_libraries.title')" 
    :width="600"
    :height="400"
    @cancel="clickCancel"
  >
    <div class="flex flex-col flex-1 min-h-0 border border-base-content/5 bg-base-300/30 shadow-sm rounded-box overflow-hidden relative">
      <div class="flex items-center px-3 py-2 shrink-0">
        <span class="flex-1 sidebar-panel-header-title text-base-content/30">{{ $t('msgbox.manage_libraries.libraries') }}</span>
        <TButton
          :icon="IconAdd"
          :buttonSize="'small'"
          :disabled="isMaxLibraryReached || showAddInput || isRenaming || isAddingLibrary"
          :tooltip="$t('msgbox.manage_libraries.add_new')"
          @click="startAddLibrary"
        />
      </div>

      <div class="flex-1 min-h-0 overflow-x-hidden overflow-y-auto select-none">
        <VueDraggable 
          v-model="libraries" 
          class="p-1"
          :animation="200"
          handle=".drag-handle"
          :disabled="showAddInput || isRenaming"
          @end="onDragEnd"
        >
        <div 
          v-for="lib in libraries" 
          :key="lib.id"
          :ref="(el) => setLibraryItemRef(el, lib.id)"
          class="flex items-center px-1 h-12 rounded-box group transition-all duration-200 ease-in-out"
          :class="[
            selectedLibraryId === lib.id
              ? 'text-base-content bg-base-100 hover:bg-base-100 selected-item'
              : 'text-base-content/70 hover:bg-base-100/30',
            showAddInput || (isRenaming && editingId !== lib.id) ? 'opacity-50' : 'cursor-pointer',
          ]"
          @click="selectLibrary(lib)"
        >
          <div
            class="drag-handle invisible shrink-0 cursor-move"
            :class="[
              { 'visible!': selectedLibraryId === lib.id },
              { 'cursor-not-allowed opacity-30': showAddInput || isRenaming },
            ]"
            :title="$t('msgbox.manage_libraries.reorder')"
          >
            <IconDragHandle class="w-4 h-4 text-base-content/70 hover:text-base-content" />
          </div>

          <!-- Name & Info -->
          <div class="h-full p-1 min-w-0 flex-1 flex flex-col justify-center">
            <div class="flex items-center gap-2">
              <input
                v-if="editingId === lib.id"
                :ref="(el) => setEditInputRef(el, lib.id)"
                v-model="inputNameValue"
                type="text"
                class="input input-sm w-full min-w-0"
                maxlength="64"
                @blur="saveRename(lib)"
                @keydown.enter.prevent="saveRename(lib)"
                @keydown.esc.stop="cancelRename"
                @click.stop
              />
              <div v-else class="min-w-0 flex items-center">
                <IconStack
                  class="w-4 h-4 mr-2 shrink-0"
                  :class="lib.id === currentLibraryId ? 'text-primary' : lib.hidden ? 'text-base-content/30' : 'text-base-content/70'"
                />
                <span class="truncate" 
                  :class="{ 
                    'text-primary': lib.id === currentLibraryId,
                    'text-base-content/30': lib.hidden,
                  }"
                >
                  {{ lib.name }}
                </span>
                <span v-if="lib.id === 'default'" class="ml-2 shrink-0 rounded-box border border-base-content/5 px-1.5 text-[10px] font-bold uppercase tracking-wide text-base-content/30">{{ $t('msgbox.manage_libraries.default') }}</span>
                <span v-if="lib.hidden" class="ml-2 shrink-0 rounded-box border border-base-content/5 px-1.5 text-[10px] font-bold uppercase tracking-wide text-base-content/30">{{ $t('msgbox.manage_libraries.hidden') }}</span>
              </div>
            </div>
          </div>

          <div
            class="w-40 shrink-0 text-right text-xs text-base-content/30 truncate"
            :class="{ hidden: selectedLibraryId === lib.id }"
          >
            <span v-if="libraryStats[lib.id]">
              {{ $t('statusbar.files_summary', { count: libraryStats[lib.id].fileCount.toLocaleString(), size: formatFileSize(libraryStats[lib.id].totalSize) }) }}
            </span>
            <span v-else-if="libraryStatsLoading[lib.id]">
              {{ $t('msgbox.manage_libraries.calculating_stats') }}
            </span>
            <span v-else-if="libraryStatsError[lib.id]">
              {{ $t('msgbox.manage_libraries.unable_to_load') }}
            </span>
          </div>

          <!-- Actions -->
          <div
            class="hidden w-40 shrink-0 justify-end text-base-content/70"
            :class="{ 'flex!': selectedLibraryId === lib.id }"
          >
            <TButton
              :icon="IconEdit"
              :buttonSize="'small'"
              :disabled="showAddInput || isRenaming"
              :tooltip="$t('msgbox.manage_libraries.rename')"
              @click.stop="startRename(lib)"
            />
            <TButton
              :icon="lib.hidden ? IconHide : IconUnhide"
              :buttonSize="'small'"
              :disabled="lib.id === 'default' || showAddInput || isRenaming"
              :tooltip="lib.hidden ? $t('msgbox.manage_libraries.show') : $t('msgbox.manage_libraries.hide')"
              @click.stop="toggleVisibility(lib)"
            />
            <TButton
              :icon="IconTrash"
              :buttonSize="'small'"
              :disabled="lib.id === 'default' || showAddInput || isRenaming"
              :tooltip="$t('msgbox.manage_libraries.delete')"
              @click.stop="confirmDelete(lib)"
            />
          </div>
          </div>
        </VueDraggable>

        <div v-if="showAddInput" class="flex items-center h-12 px-2 gap-2">
          <div class="w-4 shrink-0"></div>
          <input
            ref="addInputRef"
            v-model="newLibraryName"
            type="text"
            class="input input-sm flex-1 min-w-0"
            maxlength="64"
            :placeholder="$t('msgbox.manage_libraries.placeholder')"
            :disabled="isAddingLibrary"
            @blur="onAddInputBlur"
            @keydown.enter.prevent="doAddLibrary"
            @keydown.esc.stop="cancelAddLibrary"
          />
        </div>
      </div>
    </div>

    <div v-if="inputErrorMessage" class="shrink-0 px-4 pt-3 text-xs text-error leading-5">
      {{ inputErrorMessage }}
    </div>

    <div class="flex justify-end items-center shrink-0 pt-2 min-h-[56px]">
      <button
        class="t-button-primary"
        :disabled="showAddInput || isRenaming"
        @click="clickOk"
      >
        {{ $t('msgbox.manage_libraries.switch_to') }}
      </button>
    </div>

    <MessageBox
      v-if="showDeleteConfirm"
      :title="$t('msgbox.remove_library.title')"
      :message="$t('msgbox.remove_library.content', { library: libraryToDelete?.name })"
      :OkText="$t('msgbox.remove_library.ok')"
      :cancelText="$t('msgbox.cancel')"
      :warningOk="true"
      @ok="doDeleteLibrary"
      @cancel="showDeleteConfirm = false"
    />
  </ModalDialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { VueDraggable } from 'vue-draggable-plus';

import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { 
  getAppConfig, 
  addLibrary, 
  editLibrary, 
  removeLibrary, 
  hideLibrary, 
  reorderLibraries, 
  getLibraryInfo,
  switchLibrary,
} from '@/common/api';
import { isValidFileName, formatFileSize } from '@/common/utils';
import ModalDialog from '@/components/ModalDialog.vue';
import TButton from '@/components/TButton.vue';
import MessageBox from '@/components/MessageBox.vue';
import {
  IconDragHandle,
  IconEdit,
  IconTrash,
  IconHide,
  IconUnhide,
  IconAdd,
  IconStack,
} from '@/common/icons';

const props = defineProps({
  isNewLibrary: { type: Boolean, default: false },
});

const emit = defineEmits(['ok', 'cancel', 'updated']);
const uiStore = useUIStore();
const { t, locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const libraries = ref<any[]>([]);
const currentLibraryId = ref('');
const editingId = ref<string | null>(null);
const inputNameValue = ref('');
const newLibraryName = ref('');
const showAddInput = ref(false);
const inputErrorMessage = ref('');
const libraryStats = ref<Record<string, any>>({});
const libraryStatsLoading = ref<Record<string, boolean>>({});
const libraryStatsError = ref<Record<string, boolean>>({});
const isAddingLibrary = ref(false);
const selectedLibraryId = ref('');
let statsLoadToken = 0;

const isRenaming = computed(() => !!editingId.value);

const isMaxLibraryReached = computed(() => {
  const max = (config as any).main?.maxLibraryCount || 10;
  return libraries.value.length >= max;
});

// Delete Confirmation
const showDeleteConfirm = ref(false);
const libraryToDelete = ref<any>(null);

// Refs
const addInputRef = ref<HTMLInputElement | null>(null);
const editInputRefs = ref<Record<string, HTMLInputElement>>({});
const libraryItemRefs = ref<Record<string, HTMLElement>>({});

const setEditInputRef = (el: any, id: string) => {
  if (el) {
    editInputRefs.value[id] = el as HTMLInputElement;
  }
};

const setLibraryItemRef = (el: any, id: string) => {
  if (el) {
    libraryItemRefs.value[id] = el as HTMLElement;
  } else {
    delete libraryItemRefs.value[id];
  }
};

watch(newLibraryName, (val) => {
  const name = val.trim();
  if (name && !isValidFileName(name)) {
    inputErrorMessage.value = localeMsg.value.msgbox.input.file_name_invalid;
  } else {
    inputErrorMessage.value = '';
  }
});

watch(inputNameValue, (val) => {
  if (!editingId.value) return;
  const name = val.trim();
  inputErrorMessage.value = name && !isValidFileName(name)
    ? localeMsg.value.msgbox.input.file_name_invalid
    : '';
});

const onKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    // Close dialog if not in a sub-state (add input, edit input, or delete confirm)
    if (!showAddInput.value && !editingId.value && !showDeleteConfirm.value) {
      clickCancel();
    }
  }

  if (e.key === 'Enter') {
    // Keep Enter as the dialog confirm shortcut when the user is not
    // actively typing into an editable field inside the modal.
    if (showDeleteConfirm.value) return;

    const target = e.target as HTMLElement | null;
    if (!target) return;

    const tagName = target.tagName.toLowerCase();
    const isEditable =
      tagName === 'input' ||
      tagName === 'textarea' ||
      tagName === 'select' ||
      target.isContentEditable;

    if (isEditable) return;

    e.preventDefault();
    clickOk();
  }
};

onMounted(async () => {
  uiStore.pushInputHandler('ManageLibraries');
  window.addEventListener('keydown', onKeyDown);
  await loadLibraries();
  
  // If invoked as "New Library", show add input immediately
  if (props.isNewLibrary) {
    startAddLibrary();
  }
});

onUnmounted(() => {
  uiStore.removeInputHandler('ManageLibraries');
  window.removeEventListener('keydown', onKeyDown);
});

const loadLibraries = async () => {
  const appConfig = await getAppConfig();
  if (!appConfig) return;

  libraries.value = appConfig.libraries || [];
  currentLibraryId.value = appConfig.current_library_id;
  if (!selectedLibraryId.value || !libraries.value.some(lib => lib.id === selectedLibraryId.value)) {
    selectedLibraryId.value = currentLibraryId.value;
  }
  syncLibraryStatsState();

  // Let the dialog render first, then compute each library's stats in the background.
  window.requestAnimationFrame(() => {
    void loadLibraryStats(libraries.value);
  });
};

const syncLibraryStatsState = () => {
  const validIds = new Set(libraries.value.map(lib => lib.id));
  libraryStats.value = Object.fromEntries(
    Object.entries(libraryStats.value).filter(([id]) => validIds.has(id))
  );
  libraryStatsLoading.value = Object.fromEntries(
    Object.entries(libraryStatsLoading.value).filter(([id]) => validIds.has(id))
  );
  libraryStatsError.value = Object.fromEntries(
    Object.entries(libraryStatsError.value).filter(([id]) => validIds.has(id))
  );
};

const loadLibraryStats = async (libs: any[]) => {
  const loadToken = ++statsLoadToken;

  libs.forEach((lib) => {
    if (libraryStats.value[lib.id] || libraryStatsLoading.value[lib.id]) return;

    libraryStatsLoading.value = {
      ...libraryStatsLoading.value,
      [lib.id]: true,
    };
    libraryStatsError.value = {
      ...libraryStatsError.value,
      [lib.id]: false,
    };

    getLibraryInfo(lib.id)
      .then((info) => {
        if (loadToken !== statsLoadToken) return;
        if (!info) {
          libraryStatsError.value = {
            ...libraryStatsError.value,
            [lib.id]: true,
          };
          return;
        }
        libraryStats.value = {
          ...libraryStats.value,
          [lib.id]: info,
        };
      })
      .catch((error) => {
        console.error(error);
        if (loadToken !== statsLoadToken) return;
        libraryStatsError.value = {
          ...libraryStatsError.value,
          [lib.id]: true,
        };
      })
      .finally(() => {
        if (loadToken !== statsLoadToken) return;
        libraryStatsLoading.value = {
          ...libraryStatsLoading.value,
          [lib.id]: false,
        };
      });
  });
};

// --- Actions ---

const startRename = (lib: any) => {
  inputErrorMessage.value = '';
  editingId.value = lib.id;
  inputNameValue.value = lib.name;
  nextTick(() => {
    const input = editInputRefs.value[lib.id];
    if (input) input.focus();
  });
};

const cancelRename = () => {
  editingId.value = null;
  inputNameValue.value = '';
  inputErrorMessage.value = '';
};

const saveRename = async (lib: any) => {
  if (!editingId.value) return;
  const newName = inputNameValue.value.trim();
  
  if (newName === lib.name) {
    cancelRename();
    return;
  }
  
  if (!newName || !isValidFileName(newName)) {
    inputErrorMessage.value = localeMsg.value.msgbox.input.file_name_invalid;
    nextTick(() => editInputRefs.value[lib.id]?.focus());
    return;
  }

  try {
    await editLibrary(lib.id, newName);
    lib.name = newName;
    emit('updated', { type: 'rename', id: lib.id, name: newName });
    cancelRename();
  } catch (error) {
    console.error(error);
  }
};

const doAddLibrary = async () => {
  const name = newLibraryName.value.trim();
  if (!name || isAddingLibrary.value) return;
  
  if (!isValidFileName(name)) {
    inputErrorMessage.value = localeMsg.value.msgbox.input.file_name_invalid;
    nextTick(() => addInputRef.value?.focus());
    return;
  }

  try {
    isAddingLibrary.value = true;
    const newLib = await addLibrary(name);
    if (newLib) {
      newLibraryName.value = '';
      showAddInput.value = false;
      inputErrorMessage.value = '';
      await loadLibraries();
      emit('updated');
      selectedLibraryId.value = newLib.id;
      await focusLibrary(newLib.id);
    }
  } catch (error: any) {
    inputErrorMessage.value = error.message || error.toString();
  } finally {
    isAddingLibrary.value = false;
  }
};

const startAddLibrary = () => {
  if (isMaxLibraryReached.value || showAddInput.value || isRenaming.value || isAddingLibrary.value) return;
  inputErrorMessage.value = '';
  showAddInput.value = true;
  nextTick(() => addInputRef.value?.focus());
};

const onAddInputBlur = () => {
  if (isAddingLibrary.value) return;
  if (!newLibraryName.value.trim()) {
    cancelAddLibrary();
    return;
  }
  void doAddLibrary();
};

const cancelAddLibrary = () => {
  if (isAddingLibrary.value) return;
  showAddInput.value = false;
  newLibraryName.value = '';
  inputErrorMessage.value = '';
};

const focusLibrary = async (libraryId: string) => {
  await nextTick();
  libraryItemRefs.value[libraryId]?.scrollIntoView({
    behavior: 'smooth',
    block: 'nearest',
  });
};

const selectLibrary = async (lib: any) => {
  if (showAddInput.value || isRenaming.value || editingId.value === lib.id) return;
  selectedLibraryId.value = lib.id;
  await focusLibrary(lib.id);
};

const toggleVisibility = async (lib: any) => {
  const newHidden = !lib.hidden;
  try {
    await hideLibrary(lib.id, newHidden);
    lib.hidden = newHidden;
    emit('updated');
  } catch (error) {
    console.error(error);
  }
};

const confirmDelete = (lib: any) => {
  libraryToDelete.value = lib;
  showDeleteConfirm.value = true;
};

const doDeleteLibrary = async () => {
  if (!libraryToDelete.value) return;
  try {
    const deletedId = libraryToDelete.value.id;
    const wasCurrent = deletedId === currentLibraryId.value;
    await removeLibrary(deletedId);
    showDeleteConfirm.value = false;
    libraryToDelete.value = null;
    await loadLibraries();
    emit('updated');

    // If we just deleted the active library, the backend has already switched
    // current_library_id. Emit 'ok' so Home.vue reloads the new library's state.
    if (wasCurrent) {
      selectedLibraryId.value = currentLibraryId.value;
      emit('ok', { type: 'switch', id: currentLibraryId.value });
    }
  } catch (error) {
    console.error(error);
  }
};

const clickOk = async () => {
  if (selectedLibraryId.value && selectedLibraryId.value !== currentLibraryId.value) {
    try {
      await switchLibrary(selectedLibraryId.value);
      emit('ok', { type: 'switch', id: selectedLibraryId.value });
      return;
    } catch (error) {
      console.error(error);
      return;
    }
  }
  emit('cancel');
};

const clickCancel = () => {
  emit('cancel');
};

// --- Drag and Drop ---

const onDragEnd = async () => {
  // Persist order
  const ids = libraries.value.map(l => l.id);
  try {
    await reorderLibraries(ids);
    emit('updated', { type: 'reorder', ids });
  } catch (error) {
    console.error(error);
  }
};

</script>

<style scoped>
.ghost {
  opacity: 0.5;
  background: var(--base-200);
}
</style>

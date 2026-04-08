<template>
  <ModalDialog 
    :title="$t('msgbox.manage_libraries.title')" 
    :width="600"
    :height="400"
    @cancel="clickCancel"
  >
    <!-- Library list -->
    <div class="flex flex-col flex-1 min-h-0 border border-base-content/10 bg-base-300/30 shadow-sm rounded-box overflow-hidden relative">
      <!-- Header -->
      <div class="flex items-center justify-between px-4 pt-2 text-sm text-base-content/30 border-base-content/10 mr-9">
        <div>{{ $t('msgbox.manage_libraries.name') }}</div>
        <div class="text-right">{{ $t('msgbox.manage_libraries.action') }}</div>
      </div>

      <VueDraggable 
        v-model="libraries" 
        class="flex-1 overflow-x-hidden overflow-y-auto p-1 rounded-box select-none"
        :animation="200"
        handle=".drag-handle"
        :disabled="showAddInput || isRenaming"
        @end="onDragEnd"
      >
        <div 
          v-for="(lib, index) in libraries" 
          :key="lib.id"
          :ref="(el) => setLibraryItemRef(el, lib.id)"
          class="flex items-center justify-between mx-1 px-1 h-12 rounded-box group transition-all duration-200 ease-in-out"
          :class="[
            selectedLibraryId === lib.id
              ? 'text-base-content bg-base-100 hover:bg-base-100 selected-item'
              : 'text-base-content/70 hover:bg-base-100/30',
            showAddInput || (isRenaming && editingId !== lib.id) ? 'opacity-50' : 'cursor-pointer',
          ]"
          @click="selectLibrary(lib)"
        >
          <!-- Name & Info -->
          <div class="p-1 min-w-0 flex flex-col justify-center">
            <div class="flex items-center gap-2">
              <input
                v-if="editingId === lib.id"
                :ref="(el) => setEditInputRef(el, lib.id)"
                v-model="inputNameValue"
                type="text"
                class="input w-full h-6"
                maxlength="32"
                @blur="saveRename(lib)"
                @keydown.enter.prevent="saveRename(lib)"
                @keydown.esc.stop="cancelRename"
                @click.stop
              />
              <div v-else class="min-w-0 flex items-center">
                <span class="truncate cursor-default" 
                  :class="{ 
                    'text-primary': lib.id === currentLibraryId,
                    'text-base-content/30': lib.hidden,
                  }"
                >
                  {{ lib.name }}
                </span>
                <span v-if="lib.id === 'default'" class="shrink-0 text-xs px-2 py-1 ml-2 rounded-box bg-base-100/30">{{ $t('msgbox.manage_libraries.default') }}</span>
                <span v-if="lib.hidden" class="shrink-0 text-xs px-2 py-1 ml-2 rounded-box bg-base-100/30">{{ $t('msgbox.manage_libraries.hidden') }}</span>
              </div>
            </div>
            <div class="text-xs text-base-content/30 truncate">
              <span v-if="libraryStats[lib.id]">
                {{ $t('statusbar.files_summary', { count: libraryStats[lib.id].fileCount.toLocaleString(), size: formatFileSize(libraryStats[lib.id].totalSize) }) }}
                {{ ', '  + $t('msgbox.manage_libraries.created_at_lower') + ' ' + formatTimestamp(lib.created_at, t('format.date_time')) }}
              </span>
              <span v-else-if="libraryStatsLoading[lib.id]">
                {{ $t('msgbox.manage_libraries.calculating_stats') }}
                {{ ', '  + $t('msgbox.manage_libraries.created_at_lower') + ' ' + formatTimestamp(lib.created_at, t('format.date_time')) }}
              </span>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex text-base-content/70">
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
            <div class="drag-handle cursor-move" :class="{ 'cursor-not-allowed opacity-50': showAddInput || isRenaming }">
              <TButton
                :icon="IconDragHandle"
                :buttonSize="'small'"
                :disabled="showAddInput || isRenaming"
                :tooltip="$t('msgbox.manage_libraries.reorder')"
              />
            </div>
          </div>
        </div>
      </VueDraggable>
    </div>

    <!-- button area -->
    <div class="flex justify-between items-center shrink-0 pt-2 min-h-[64px]">
      <!-- Add New Library -->
      <div class="flex flex-col items-start justify-center p-2 w-2/3 min-h-[48px] rounded-box border border-transparent transition-colors" :class="showAddInput ? 'border-base-content/10 bg-base-100/20' : ''">
        <button
          v-if="!showAddInput" 
          class="inline-flex h-8 items-center gap-2 px-3 py-2 rounded-box border border-base-content/10 text-sm transition-colors"
          :class="isMaxLibraryReached || isRenaming
            ? 'text-base-content/30 cursor-default'
            : 'text-base-content/70 hover:bg-base-100/30 hover:text-base-content cursor-pointer'"
          :title="isMaxLibraryReached ? $t('msgbox.manage_libraries.max_limit_reached') : $t('msgbox.manage_libraries.add_new')"
          :disabled="isMaxLibraryReached || isRenaming"
          @click="showAddInput = true"
        >
          <IconAdd class="w-5 h-5" />
          <span>{{ $t('msgbox.manage_libraries.add_new') }}</span>
        </button>
        <template v-else>
          <div class="w-full flex min-h-8 items-center gap-2">
            <input
              ref="addInputRef"
              v-model="newLibraryName"
              type="text"
              class="input input-sm flex-1 min-w-0"
              maxlength="32"
              :placeholder="$t('msgbox.manage_libraries.placeholder')"
              :disabled="isAddingLibrary"
              @keydown.enter="doAddLibrary"
              @keydown.esc.stop="cancelAddLibrary"
            />
            <button
              class="px-3 py-1 rounded-box text-sm transition-colors shrink-0"
              :class="isAddingLibrary ? 'text-base-content/30 cursor-default' : 'text-base-content/70 hover:bg-base-100/30 hover:text-base-content cursor-pointer'"
              :disabled="isAddingLibrary"
              @click="cancelAddLibrary"
            >
              {{ $t('msgbox.cancel') }}
            </button>
            <button
              class="px-3 py-1 rounded-box text-sm transition-colors shrink-0"
              :class="canSubmitNewLibrary ? 'bg-primary text-primary-content hover:opacity-90 cursor-pointer' : 'bg-base-100/40 text-base-content/30 cursor-default'"
              :disabled="!canSubmitNewLibrary"
              @click="doAddLibrary"
            >
              {{ isAddingLibrary ? $t('tooltip.loading') : $t('msgbox.manage_libraries.add') }}
            </button>
          </div>
        </template>
      </div>
      <div class="w-0 flex-1 px-2 pt-1 text-error text-xs leading-5">
        {{ inputErrorMessage }}
      </div>

      <button
        class="px-4 py-1 rounded-box text-base-content/70 hover:bg-primary hover:text-base-100 cursor-pointer shrink-0"
        @click="clickOk"
      >
        {{ $t('msgbox.ok') }}
      </button>
    </div>

    <!-- Inner Message Box for Delete Confirmation -->
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
import { formatTimestamp, isValidFileName, formatFileSize } from '@/common/utils';
import ModalDialog from '@/components/ModalDialog.vue';
import TButton from '@/components/TButton.vue';
import MessageBox from '@/components/MessageBox.vue'; // Need to import or ensure it is available
import {
  IconDragHandle,
  IconEdit,
  IconTrash,
  IconHide,
  IconUnhide,
  IconAdd,
} from '@/common/icons';

// Props are less relevant now but kept for compatibility logic if needed
const props = defineProps({
  isNewLibrary: { type: Boolean, default: false },
});

const emit = defineEmits(['ok', 'cancel']);
const uiStore = useUIStore();
const { t, locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

// State
const libraries = ref<any[]>([]);
const currentLibraryId = ref('');
const editingId = ref<string | null>(null);
const inputNameValue = ref('');
const newLibraryName = ref('');
const showAddInput = ref(false);
const inputErrorMessage = ref('');
const libraryStats = ref<Record<string, any>>({});
const libraryStatsLoading = ref<Record<string, boolean>>({});
const isAddingLibrary = ref(false);
const selectedLibraryId = ref('');
let statsLoadToken = 0;

const isRenaming = computed(() => !!editingId.value);
const canSubmitNewLibrary = computed(() => !!newLibraryName.value.trim() && !inputErrorMessage.value && !isAddingLibrary.value);

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

watch(showAddInput, (newValue) => {
  if (newValue) {
    // Reset error when showing input
    inputErrorMessage.value = '';
    // Also focus input
    nextTick(() => addInputRef.value?.focus());
  } else {
    cancelAddLibrary();
  }
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
    showAddInput.value = true;
    nextTick(() => addInputRef.value?.focus());
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
};

const loadLibraryStats = async (libs: any[]) => {
  const loadToken = ++statsLoadToken;

  libs.forEach((lib) => {
    if (libraryStats.value[lib.id] || libraryStatsLoading.value[lib.id]) return;

    libraryStatsLoading.value = {
      ...libraryStatsLoading.value,
      [lib.id]: true,
    };

    getLibraryInfo(lib.id)
      .then((info) => {
        if (loadToken !== statsLoadToken || !info) return;
        libraryStats.value = {
          ...libraryStats.value,
          [lib.id]: info,
        };
      })
      .catch((error) => {
        console.error(error);
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
};

const saveRename = async (lib: any) => {
  if (!editingId.value) return;
  const newName = inputNameValue.value.trim();
  
  if (newName === lib.name) {
    cancelRename();
    return;
  }
  
  if (!newName || !isValidFileName(newName)) {
    // Ideally show toast or small error, but for now just cancel if invalid
    // or maybe shake input
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
      selectedLibraryId.value = newLib.id;
      await focusLibrary(newLib.id);
    }
  } catch (error: any) {
    inputErrorMessage.value = error.message || error.toString();
  } finally {
    isAddingLibrary.value = false;
  }
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
    await removeLibrary(libraryToDelete.value.id);
    showDeleteConfirm.value = false;
    libraryToDelete.value = null;
    await loadLibraries();
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

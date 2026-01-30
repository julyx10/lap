<template>
  <ModalDialog 
    :title="$t('msgbox.manage_libraries.title')" 
    :width="600"
    :height="400"
    @cancel="clickClose"
  >
    <!-- Library list -->
    <div class="flex flex-col flex-1 min-h-0 border border-base-content/10 rounded-box overflow-hidden relative">
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
          class="flex items-center justify-between mx-1 px-1 h-12 rounded-box transition-colors group"
          :class="{ 
            'bg-base-100/30': editingId === lib.id,
            'hover:bg-base-100/30': !showAddInput && !isRenaming, 
            'opacity-50': showAddInput || (isRenaming && editingId !== lib.id) 
          }"
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
                @keydown.enter="saveRename(lib)"
                @keydown.esc.stop="cancelRename"
                @click.stop
              />
              <div v-else class="min-w-0 flex items-center">
                <span class="truncate cursor-default" 
                  :class="{ 
                    'text-primary': lib.id === currentLibraryId,
                  }"
                >
                  {{ lib.name }}
                </span>
                <span v-if="lib.id === 'default'" class="shrink-0 text-xs px-2 py-1 ml-2 rounded-box bg-base-100/30">{{ $t('msgbox.manage_libraries.default') }}</span>
                <span v-if="lib.hidden" class="shrink-0 text-xs px-2 py-1 ml-2 rounded-box bg-base-100/30">{{ $t('msgbox.manage_libraries.hide') }}</span>
              </div>
            </div>
            <div class="text-xs text-base-content/30 truncate">
              <span v-if="libraryStats[lib.id]">
                {{ formatFileSize(libraryStats[lib.id].db_file_size || 0) }}, {{ $t('msgbox.manage_libraries.created_at_lower') }} {{ formatTimestamp(lib.created_at, t('format.date_time')) }}
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
              :disabled="lib.id === 'default' || lib.id === currentLibraryId || showAddInput || isRenaming"
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
    <div class="flex justify-between items-center shrink-0 pt-2">
      <!-- Add New Library -->
      <div class="flex items-center gap-2 p-2 w-2/3 rounded-box" :class="showAddInput ? ' border border-base-content/10' : 'border border-transparent'">
        <TButton 
          v-if="!showAddInput" 
          :icon="IconAdd" 
          :buttonSize="'medium'" 
          :tooltip="isMaxLibraryReached ? $t('msgbox.manage_libraries.max_limit_reached') : $t('msgbox.manage_libraries.add_new')" 
          :disabled="isMaxLibraryReached || isRenaming"
          @click="showAddInput = true"
        />
        <template v-else>
          <input
            ref="addInputRef"
            v-model="newLibraryName"
            type="text"
            class="input input-sm w-full"
            maxlength="32"
            :placeholder="$t('msgbox.manage_libraries.placeholder')"
            @keydown.enter="doAddLibrary"
            @keydown.esc.stop="showAddInput = false"
          />
          <div class="flex items-center gap-1">
            <TButton
              :icon="IconClose"
              :tooltip="$t('msgbox.cancel')"
              @click="showAddInput = false"
            />
            <TButton
              :icon="IconOk"
              :tooltip="$t('msgbox.manage_libraries.add')"
              :disabled="!newLibraryName.trim()"
              @click="doAddLibrary"
            />
          </div>
        </template>
      </div>
      <div v-if="inputErrorMessage" class="text-error text-xs mt-1 px-2">{{ inputErrorMessage }}</div>

      <button 
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer shrink-0" 
        @click="clickClose"
      >
        {{ $t('msgbox.close') }}
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
  getLibraryInfo 
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
  IconOk,
  IconClose,
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

const setEditInputRef = (el: any, id: string) => {
  if (el) {
    editInputRefs.value[id] = el as HTMLInputElement;
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
    newLibraryName.value = '';
    inputErrorMessage.value = '';
  }
});

const onKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    // Close dialog if not in a sub-state (add input, edit input, or delete confirm)
    if (!showAddInput.value && !editingId.value && !showDeleteConfirm.value) {
      clickClose();
    }
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
  const config = await getAppConfig();
  if (config) {
    libraries.value = config.libraries || [];
    currentLibraryId.value = config.current_library_id;
    
    // Lazy load stats (files count, size, path)
    for (const lib of libraries.value) {
       if (!libraryStats.value[lib.id]) {
          getLibraryInfo(lib.id).then(info => {
             if (info) libraryStats.value[lib.id] = info;
          });
       }
    }
  }
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
    cancelRename();
    emit('ok', { type: 'rename', id: lib.id, name: newName }); // Notify parent to refresh if needed
  } catch (error) {
    console.error(error);
  }
};

const doAddLibrary = async () => {
  const name = newLibraryName.value.trim();
  if (!name) return;
  
  if (!isValidFileName(name)) {
    inputErrorMessage.value = localeMsg.value.msgbox.input.file_name_invalid;
    return;
  }

  try {
    const newLib = await addLibrary(name);
    if (newLib) {
      newLibraryName.value = '';
      showAddInput.value = false;
      inputErrorMessage.value = '';
      await loadLibraries();
      emit('ok', { type: 'add', library: newLib });
    }
  } catch (error: any) {
    inputErrorMessage.value = error.message || error.toString();
  }
};

const toggleVisibility = async (lib: any) => {
  if (lib.id === currentLibraryId.value) return; // Cannot hide current
  
  const newHidden = !lib.hidden;
  try {
    await hideLibrary(lib.id, newHidden);
    lib.hidden = newHidden;
    emit('ok', { type: 'hide' }); // Refresh menu
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
    emit('ok', { type: 'delete' });
  } catch (error) {
    console.error(error);
  }
};

const clickClose = () => {
  emit('cancel'); // Just close dialog
};

// --- Drag and Drop ---

const onDragEnd = async () => {
  // Persist order
  const ids = libraries.value.map(l => l.id);
  try {
    await reorderLibraries(ids);
    emit('ok', { type: 'reorder' });
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

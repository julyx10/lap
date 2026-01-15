<template>
  <ModalDialog 
    :title="isNewLibrary ? $t('msgbox.edit_library.title_add') : $t('msgbox.edit_library.title_edit')" 
    @cancel="clickCancel"
  >
    <!-- two column grid layout -->
    <div class="w-full text-sm text-nowrap grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 items-center">
      <!-- Name -->
      <div class="h-8 flex items-center">{{ $t('msgbox.edit_library.name') }}</div>
      <div class="flex items-center">
        <input
          ref="inputNameRef"
          v-model="inputNameValue"
          type="text"
          maxlength="255"
          class="w-full input"
          :placeholder="$t('msgbox.edit_library.placeholder')"
          @input="validateInput"
        />
      </div>
    </div>
    
    <!-- Error message (always visible line for consistent layout) -->
    <p class="h-6 w-full flex items-center justify-center p-2 text-error text-xs">{{ inputErrorMessage }}</p>

    <!-- library info -->
    <div v-if="!isNewLibrary" class="w-full text-sm text-nowrap grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 items-center">
      <div class="flex items-center">{{ $t('msgbox.edit_library.library_file_path') }}</div>
      <div class="flex items-center text-base-content/70 text-wrap overflow-hidden">
        {{ libInfo?.db_file_path || '' }}
      </div>
      
      <div class="flex items-center">{{ $t('msgbox.edit_library.library_file_size') }}</div>
      <div class="flex items-center text-base-content/70">
        {{ formatFileSize(libInfo?.db_file_size || 0) }}
      </div>
      
      <div class="flex items-center">{{ $t('msgbox.edit_library.created_at') }}</div>
      <div class="flex items-center text-base-content/70">
        {{ formattedCreatedAt }}
      </div>
    </div>

    <!-- cancel and OK buttons -->
    <div class="mt-2 flex justify-end space-x-4">
      <button 
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer" 
        @click="clickCancel"
      >
        {{ $t('msgbox.cancel') }}
      </button>
      <button 
        :class="[
          'px-4 py-1 rounded-box', 
          isValidName && !inputErrorMessage ? 'hover:bg-primary hover:text-base-100 cursor-pointer' : 'text-base-content/30 cursor-default',
        ]" 
        @click="clickOk"
      >
        {{ $t('msgbox.ok') }}
      </button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { getLibraryInfo, addLibrary, editLibrary } from '@/common/api';
import { formatFileSize, formatTimestamp, isValidFileName } from '@/common/utils';
import { useUIStore } from '@/stores/uiStore';

import ModalDialog from '@/components/ModalDialog.vue';

interface LibraryInfo {
  db_file_size: number;
  db_file_path: string;
}

const props = defineProps({
  isNewLibrary: {
    type: Boolean,
    default: false
  },
  libraryId: {
    type: String,
    default: ''
  },
  libraryName: { 
    type: String, 
    default: '' 
  },
  createdAt: { 
    type: Number, 
    default: 0 
  },
});

const emit = defineEmits(['ok', 'cancel']);
const uiStore = useUIStore();
const { t, locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

// input 
const inputNameRef = ref<HTMLInputElement | null>(null);
const inputNameValue = ref(props.libraryName);
const libInfo = ref<LibraryInfo | null>(null);
const inputErrorMessage = ref('');

// Validate library name
const isValidName = computed(() => {
  const name = inputNameValue.value.trim();
  return name.length > 0;
});

// Validate input on each keystroke
const validateInput = () => {
  const name = inputNameValue.value.trim();
  if (name.length === 0) {
    inputErrorMessage.value = '';
  } else if (!isValidFileName(name)) {
    inputErrorMessage.value = localeMsg.value.msgbox.input.file_name_invalid;
  } else {
    inputErrorMessage.value = '';
  }
};

// Format created at timestamp
const formattedCreatedAt = computed(() => {
  if (props.createdAt) {
    return formatTimestamp(props.createdAt, t('format.date_time'));
  }
  return '';
});

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('LibraryEdit');
  
  if (!props.isNewLibrary && props.libraryId) {
    // Load library info
    libInfo.value = await getLibraryInfo(props.libraryId);
  }

  setTimeout(() => {
    inputNameRef.value?.focus();
    inputNameRef.value?.select();
  }, 50);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('LibraryEdit');
});

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('LibraryEdit')) return;

  const { key } = event;

  switch (key) {
    case 'Enter':
      clickOk();
      break;
    case 'Escape':
      clickCancel();
      break;
    default:
      break;
  }
}

const clickOk = async () => {
  if (!isValidName.value || inputErrorMessage.value) return;
  
  const name = inputNameValue.value.trim();
  
  try {
    if (props.isNewLibrary) {
      const library = await addLibrary(name);
      if (library) {
        emit('ok', library);
      }
    } else {
      await editLibrary(props.libraryId, name);
      emit('ok', { id: props.libraryId, name });
    }
  } catch (error: any) {
    // Display backend error (e.g., duplicate name)
    inputErrorMessage.value = error.message || error.toString();
  }
};

const clickCancel = () => {
  emit('cancel');
};
</script>


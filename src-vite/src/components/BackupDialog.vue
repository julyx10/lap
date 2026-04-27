<template>
  <ModalDialog
    :title="$t('settings.database.backup_dialog_title')"
    :width="500"
    :height="380"
    @cancel="clickCancel"
  >
    <div class="flex flex-col flex-1 min-h-0 overflow-hidden">
      <div class="text-sm text-base-content/50 mb-2">
        {{ $t('settings.database.backup_select_hint') }}
      </div>

      <!-- Select all -->
      <div class="flex items-center gap-2 px-1 mb-1">
        <label class="flex items-center gap-2 cursor-pointer text-sm text-base-content/70 hover:text-base-content">
          <input type="checkbox" class="checkbox checkbox-xs checkbox-primary" :checked="allSelected" :indeterminate="someSelected && !allSelected" @change="toggleAll" />
          {{ $t('settings.database.backup_select_all') }}
        </label>
      </div>

      <!-- Library list -->
      <div class="flex-1 overflow-y-auto border border-base-content/10 bg-base-300/30 shadow-sm rounded-box p-1">
        <div
          v-for="lib in libraries"
          :key="lib.libraryId"
          class="flex items-center justify-between px-3 py-2.5 rounded-box hover:bg-base-100/20 cursor-pointer transition-colors"
          :class="{ 'bg-base-100/10': selectedIds.has(lib.libraryId) }"
          @click="toggleLib(lib.libraryId)"
        >
          <div class="flex items-center gap-2 min-w-0">
            <input type="checkbox" class="checkbox checkbox-xs checkbox-primary" :checked="selectedIds.has(lib.libraryId)" @click.stop @change="toggleLib(lib.libraryId)" />
            <span class="truncate text-sm">{{ lib.libraryName }}</span>
          </div>
          <span class="shrink-0 text-xs text-base-content/40">{{ formatFileSize(lib.dbFileSize) }}</span>
        </div>
      </div>

      <!-- Estimated size -->
      <div class="text-xs text-base-content/40 mt-2">
        {{ $t('settings.database.backup_estimated_size', { size: formatFileSize(estimatedSize) }) }}
      </div>
    </div>

    <!-- Buttons -->
    <div class="flex justify-end items-center gap-2 shrink-0 pt-2">
      <button
        class="px-4 py-1.5 rounded-box text-base-content/70 hover:bg-base-100/30 cursor-pointer text-sm"
        @click="clickCancel"
      >
        {{ $t('msgbox.cancel') }}
      </button>
      <button
        class="px-4 py-1.5 rounded-box text-sm cursor-pointer"
        :class="selectedIds.size > 0 && !isProcessing
          ? 'bg-primary text-primary-content hover:opacity-90'
          : 'bg-base-100/40 text-base-content/30 cursor-default'"
        :disabled="selectedIds.size === 0 || isProcessing"
        @click="doBackup"
      >
        {{ isProcessing ? $t('settings.database.backup_creating') : $t('settings.database.backup') }}
      </button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { getDbStorageInfo, backupDatabases } from '@/common/api';
import { formatFileSize } from '@/common/utils';
import ModalDialog from '@/components/ModalDialog.vue';
import { save } from '@tauri-apps/plugin-dialog';
import { useToast } from '@/common/toast';

const emit = defineEmits(['done', 'cancel']);
const { t, locale, messages } = useI18n();
const toast = useToast();
const localeMsg = computed(() => messages.value[locale.value] as any);

interface LibInfo {
  libraryId: string;
  libraryName: string;
  dbFileSize: number;
}

const libraries = ref<LibInfo[]>([]);
const selectedIds = ref<Set<string>>(new Set());
const isProcessing = ref(false);

const allSelected = computed(() =>
  libraries.value.length > 0 && selectedIds.value.size === libraries.value.length
);
const someSelected = computed(() => selectedIds.value.size > 0);

const estimatedSize = computed(() => {
  let total = 0;
  for (const id of selectedIds.value) {
    const lib = libraries.value.find(l => l.libraryId === id);
    if (lib) total += lib.dbFileSize;
  }
  return total;
});

const toggleLib = (id: string) => {
  const next = new Set(selectedIds.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  selectedIds.value = next;
};

const toggleAll = () => {
  if (allSelected.value) {
    selectedIds.value = new Set();
  } else {
    selectedIds.value = new Set(libraries.value.map(l => l.libraryId));
  }
};

const doBackup = async () => {
  if (selectedIds.value.size === 0) return;
  isProcessing.value = true;

  try {
    // Show save dialog
    const destPath = await save({
      title: localeMsg.value.settings?.database?.backup_dialog_title || 'Backup Libraries',
      defaultPath: `Lap-backup-${new Date().toISOString().slice(0, 10)}.zip`,
      filters: [{ name: 'ZIP Archive', extensions: ['zip'] }],
    });

    if (!destPath) {
      isProcessing.value = false;
      return;
    }

    const result = await backupDatabases(Array.from(selectedIds.value), destPath);
    toast.success(t('settings.database.backup_success', {
      path: result.filePath,
      size: formatFileSize(result.fileSize),
    }));
    emit('done');
  } catch (error: any) {
    toast.error(error.message || error.toString());
  } finally {
    isProcessing.value = false;
  }
};

const clickCancel = () => {
  if (isProcessing.value) return;
  emit('cancel');
};

// Load library info on mount
getDbStorageInfo().then((info) => {
  libraries.value = info;
  // Select all by default
  selectedIds.value = new Set(info.map(l => l.libraryId));
});
</script>

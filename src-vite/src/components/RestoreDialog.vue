<template>
  <ModalDialog
    :title="$t('settings.database.restore_dialog_title')"
    :width="520"
    :height="420"
    @cancel="clickCancel"
  >
    <div v-if="!parsed" class="flex flex-col items-center justify-center flex-1 text-base-content/50">
      <button class="mt-3 px-4 py-1.5 rounded-box bg-primary text-primary-content hover:opacity-90 cursor-pointer text-sm" @click="selectFile">
        {{ $t('settings.database.restore_select_file') }}
      </button>
    </div>

    <div v-else class="flex flex-col flex-1 min-h-0 overflow-hidden">
      <!-- Backup file info -->
      <div class="text-xs text-base-content/40 mb-2 space-y-0.5">
        <div>{{ $t('settings.database.restore_file_info', { path: backupFileName }) }}</div>
        <div>{{ $t('settings.database.restore_created_at', { date: formatTimestamp(parsed.createdAt, t('format.date_time_long')) }) }}</div>
        <div>{{ $t('settings.database.restore_libraries', { count: parsed.libraries.length, size: formatFileSize(totalBackupSize) }) }}</div>
      </div>

      <!-- Library list -->
      <div class="flex-1 overflow-y-auto border border-base-content/10 bg-base-300/30 shadow-sm rounded-box p-1">
        <div
          v-for="lib in parsed.libraries"
          :key="lib.name"
          class="flex items-center justify-between px-3 py-2.5 rounded-box hover:bg-base-100/20 cursor-pointer transition-colors"
          :class="{ 'bg-base-100/10': restoreSelections[lib.name] }"
          @click="toggleLib(lib.name)"
        >
          <div class="flex items-center gap-2 min-w-0">
            <input type="checkbox" class="checkbox checkbox-xs checkbox-primary" :checked="!!restoreSelections[lib.name]" @click.stop @change="toggleLib(lib.name)" />
            <span class="truncate text-sm">{{ lib.name }}</span>
          </div>
          <div class="shrink-0 flex items-center gap-2">
            <span
              v-if="isNewLibrary(lib.name)"
              class="text-xs px-1.5 py-0.5 rounded bg-success/10 text-success"
            >
              {{ $t('settings.database.restore_new_library') }}
            </span>
            <span
              v-else
              class="text-xs px-1.5 py-0.5 rounded bg-warning/10 text-warning"
            >
              {{ $t('settings.database.restore_conflict') }}
            </span>
            <span class="text-xs text-base-content/40 w-16 text-right">{{ formatFileSize(lib.dbSize) }}</span>
          </div>
        </div>
      </div>

      <!-- Note -->
      <div class="text-xs text-base-content/40 mt-2">
        ⚠ {{ $t('settings.database.restore_note') }}
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
        :class="parsed && restoreCount > 0 && !isProcessing
          ? 'bg-primary text-primary-content hover:opacity-90'
          : 'bg-base-100/40 text-base-content/30 cursor-default'"
        :disabled="!parsed || restoreCount === 0 || isProcessing"
        @click="doRestore"
      >
        {{ isProcessing ? $t('settings.database.restore_restoring') : $t('settings.database.restore') }}
      </button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { parseBackupFile, restoreDatabases } from '@/common/api';
import { formatFileSize, formatTimestamp } from '@/common/utils';
import ModalDialog from '@/components/ModalDialog.vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useToast } from '@/common/toast';

const emit = defineEmits(['done', 'cancel']);
const { t, locale, messages } = useI18n();
const toast = useToast();
const localeMsg = computed(() => messages.value[locale.value] as any);

interface BackupLib {
  name: string;
  dbSize: number;
}

interface ParsedBackup {
  createdAt: number;
  libraries: BackupLib[];
}

const parsed = ref<ParsedBackup | null>(null);
const backupFileName = ref('');
const backupFullPath = ref('');
const existingNames = ref<Set<string>>(new Set());
const restoreSelections = ref<Record<string, boolean>>({});
const isProcessing = ref(false);

const totalBackupSize = computed(() => {
  if (!parsed.value) return 0;
  return parsed.value.libraries.reduce((sum, lib) => sum + lib.dbSize, 0);
});

const restoreCount = computed(() =>
  Object.values(restoreSelections.value).filter(Boolean).length
);

const isNewLibrary = (name: string) => !existingNames.value.has(name);

const toggleLib = (name: string) => {
  restoreSelections.value = {
    ...restoreSelections.value,
    [name]: !restoreSelections.value[name],
  };
};

// Load existing library names for conflict detection
import { getAppConfig } from '@/common/api';
const loadExistingNames = async () => {
  try {
    const config = await getAppConfig();
    if (config?.libraries) {
      existingNames.value = new Set(config.libraries.map((l: any) => l.name));
    }
  } catch {
    // ignore
  }
};
loadExistingNames();

const selectFile = async () => {
  const filePath = await open({
    title: localeMsg.value.settings?.database?.restore_select_file || 'Select Backup File',
    filters: [{ name: 'ZIP Archive', extensions: ['zip'] }],
    multiple: false,
  });

  if (!filePath) return;

  try {
    const meta = await parseBackupFile(filePath);
    backupFullPath.value = filePath as string;
    // Extract just the filename for display
    const parts = (filePath as string).split(/[\\/]/);
    backupFileName.value = parts[parts.length - 1];

    parsed.value = {
      createdAt: meta.createdAt,
      libraries: meta.libraries,
    };

    // Default: select all libraries
    restoreSelections.value = {};
    for (const lib of meta.libraries) {
      restoreSelections.value[lib.name] = true;
    }
  } catch (error: any) {
    toast.error(error.message || error.toString());
  }
};

const doRestore = async () => {
  if (!parsed.value || restoreCount.value === 0) return;
  isProcessing.value = true;

  try {
    const selections = Object.entries(restoreSelections.value)
      .filter(([_, selected]) => selected)
      .map(([name]) => ({
        libraryName: name,
        shouldRename: !isNewLibrary(name),
      }));

    const result = await restoreDatabases(backupFullPath.value, selections);
    toast.success(t('settings.database.restore_success', {
      count: result.restoredCount,
      names: result.restoredNames.join(', '),
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
</script>

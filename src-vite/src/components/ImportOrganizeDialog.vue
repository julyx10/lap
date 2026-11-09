<template>
  <ModalDialog :title="$t('import_organize.title')" :width="520" position-key="import-organize" @cancel="close">
    <div class="space-y-2 select-none">
      <section class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
        <div class="font-bold uppercase text-[10px] tracking-widest text-base-content/30">{{ $t('import_organize.section_folders') }}</div>
        <div class="grid grid-cols-[84px_1fr] gap-x-4 gap-y-2 items-start px-1 text-xs">
          <div class="min-h-6 flex items-center text-[11px] text-base-content/30">{{ $t('import_organize.source') }}</div>
          <div class="min-w-0 min-h-6 flex items-center justify-between gap-2">
            <input v-if="sourcePath" readonly :value="sourcePath" :title="sourcePath" class="w-full bg-transparent border-none p-0 text-[12px] text-base-content/70 focus:outline-none focus:ring-0" />
            <button v-else class="btn btn-primary btn-sm rounded-box" :disabled="locked" @click="chooseSource">
              <IconDownload class="w-4 h-4" />{{ $t('album.edit.select_folder') }}
            </button>
            <TButton v-if="sourcePath" :icon="IconDownload" :selected="true" :disabled="locked" :tooltip="$t('album.edit.select_folder')" @click="chooseSource" />
          </div>

          <div class="min-h-6 flex items-center text-[11px] text-base-content/30">{{ $t('import_organize.destination') }}</div>
          <div class="min-w-0 space-y-2">
            <div class="min-w-0 min-h-6 flex items-center">
              <input readonly :value="destinationPath" :title="destinationPath" class="w-full bg-transparent border-none p-0 text-[12px] text-base-content/70 focus:outline-none focus:ring-0" />
            </div>
            <div :inert="locked" class="max-h-44 overflow-y-auto rounded-box bg-base-100/40 border border-base-content/10 p-1" :class="{ 'opacity-60': locked }">
              <AlbumFolder :children="destinationTree ? [destinationTree] : []" :albumId="Number(album.id)" :rootPath="album.path" :allowContextMenu="false" />
            </div>
          </div>

          <label class="min-h-6 flex items-center text-[11px] text-base-content/30" for="import-layout">{{ $t('import_organize.folder_layout') }}</label>
          <select id="import-layout" v-model="layout" :disabled="locked" class="select select-bordered select-xs w-full text-[12px]">
            <option value="day">{{ $t('import_organize.layout_day') }}</option>
            <option value="month">{{ $t('import_organize.layout_month') }}</option>
            <option value="year">{{ $t('import_organize.layout_year') }}</option>
            <option value="structure">{{ $t('import_organize.layout_structure') }}</option>
          </select>
        </div>
      </section>

      <section class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
        <div class="font-bold uppercase text-[10px] tracking-widest text-base-content/30">{{ $t('import_organize.section_status') }}</div>
        <div v-if="!running && !completed" class="grid grid-cols-[84px_1fr] gap-x-4 gap-y-1.5 items-center px-1 text-xs">
          <div class="min-h-6 flex items-center text-[11px] text-base-content/30">{{ $t('import_organize.to_import') }}</div>
          <div class="min-h-6 flex items-center text-[12px] text-base-content/70" :class="{ 'animate-pulse': counting }">
            <span v-if="!sourcePath">{{ $t('import_organize.source_placeholder') }}</span>
            <span v-else-if="counting">{{ $t('album.edit.files_counting') }}</span>
            <span v-else-if="sourceSummary">{{ $t('album.edit.files_count', { count: sourceSummary.count.toLocaleString(), size: formatFileSize(sourceSummary.size) }) }}</span>
            <span v-else>—</span>
          </div>
        </div>
        <div v-if="running || completed" class="space-y-2 px-1 text-xs">
          <div class="flex items-center justify-between gap-2 text-base-content/70">
            <span>{{ completed ? $t(cancelled ? 'import_organize.cancelled' : 'import_organize.complete') : progress.phase === 'preparing' ? $t('import_organize.preparing') : $t('import_organize.importing') }}</span>
            <span>{{ $t('import_organize.files_progress', { processed: progress.processed.toLocaleString(), total: progress.total.toLocaleString(), size: formatFileSize(progress.totalSize) }) }}</span>
          </div>
          <progress class="progress progress-primary w-full" :value="running && progress.phase === 'preparing' ? undefined : progress.processed" :max="Math.max(progress.total, 1)"></progress>
          <div class="flex gap-3 text-base-content/60">
            <span>{{ $t('import_organize.imported', { count: progress.imported.toLocaleString() }) }}</span>
            <span>{{ $t('import_organize.skipped', { count: progress.skipped.toLocaleString() }) }}</span>
          </div>
          <div v-if="progress.failed" class="text-error/70">{{ $t('import_organize.failed', { count: progress.failed.toLocaleString() }) }}</div>
        </div>
        <div v-if="importError" role="alert" class="px-1 text-xs text-error/70 wrap-break-word">{{ importError }}</div>
      </section>
    </div>
    <div class="mt-4 flex justify-end gap-4">
      <button v-if="!running" class="t-button-default" @click="close">{{ completed ? $t('msgbox.close') : $t('msgbox.cancel') }}</button>
      <button v-if="completed && cancelled" class="t-button-primary" @click="continueImport">{{ $t('import_organize.continue') }}</button>
      <button v-if="!running && !completed" class="t-button-primary" :disabled="!sourcePath || counting || !destinationTree" @click="startImport">{{ $t('import_organize.import') }}</button>
      <button v-if="running" class="t-button-default" :disabled="cancelling" @click="cancel">{{ $t('msgbox.cancel') }}</button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { openFolderDialog, formatFileSize } from '@/common/utils';
import { cancelImportAndOrganize, countFolder, fetchFolder, importAndOrganize, listenImportOrganizeFinished, listenImportOrganizeProgress } from '@/common/api';
import { config, libConfig } from '@/common/config';
import { useAlbumSelectionProvider } from '@/composables/useAlbumSelection';
import { listen, type Event } from '@tauri-apps/api/event';
import AlbumFolder from '@/components/AlbumFolder.vue';
import ModalDialog from '@/components/ModalDialog.vue';
import TButton from '@/components/TButton.vue';
import { IconDownload } from '@/common/icons';

const props = defineProps<{ album: any }>();
const emit = defineEmits(['complete', 'cancel']);
const uiStore = useUIStore();
const { t } = useI18n();
const sourcePath = ref('');
const sourceSummary = ref<{ count: number; size: number } | null>(null);
const counting = ref(false);
const importError = ref('');
let sourceRequest = 0;
let completedPaths: string[] = [];
let disposed = false;
const destinationTree = ref<any>(null);
const layout = ref('day');
const running = ref(false);
const completed = ref(false);
const cancelled = ref(false);
const cancelling = ref(false);
const locked = computed(() => running.value || completed.value);
const progress = ref({ phase: 'preparing', processed: 0, total: 0, totalSize: 0, imported: 0, skipped: 0, failed: 0 });
let unlistenProgress: (() => void) | undefined;
let unlistenFinished: (() => void) | undefined;
let unlistenKeydown: (() => void) | undefined;

useAlbumSelectionProvider('destFolder');
libConfig.destFolder.albumId = Number(props.album.id);
libConfig.destFolder.folderId = null;
libConfig.destFolder.folderPath = props.album.path;
libConfig.destFolder.selected = false;
const destinationPath = computed(() => String(libConfig.destFolder.folderPath || props.album.path));

async function chooseSource() {
  if (locked.value) return;
  const path = await openFolderDialog();
  if (!path || disposed || locked.value) return;
  const request = ++sourceRequest;
  sourcePath.value = path;
  completedPaths = [];
  sourceSummary.value = null;
  importError.value = '';
  counting.value = true;
  try {
    const summary = await countFolder(path);
    if (disposed || request !== sourceRequest) return;
    if (summary) sourceSummary.value = {
      count: Number(summary[1]) + Number(summary[3]),
      size: Number(summary[2]) + Number(summary[4]),
    };
  } finally {
    if (!disposed && request === sourceRequest) counting.value = false;
  }
}

async function startImport() {
  if (!sourcePath.value || locked.value || counting.value || !destinationTree.value) return;
  running.value = true;
  importError.value = '';
  progress.value = { phase: 'preparing', processed: 0, total: 0, totalSize: 0, imported: 0, skipped: 0, failed: 0 };
  try {
    unlistenProgress = await listenImportOrganizeProgress((event: any) => {
      progress.value = event.payload || progress.value;
    });
    let resolveFinished: (result: any) => void;
    let rejectFinished: (error: Error) => void;
    const finished = new Promise<any>((resolve, reject) => {
      resolveFinished = resolve;
      rejectFinished = reject;
    });
    unlistenFinished = await listenImportOrganizeFinished((event: any) => {
        const payload = event.payload || {};
        payload.error ? rejectFinished(new Error(payload.error)) : resolveFinished(payload.result);
    });
    await importAndOrganize(props.album.id, sourcePath.value, destinationPath.value, layout.value, completedPaths);
    const result = await finished;
    completedPaths = result.completedPaths;
    progress.value = { ...progress.value, ...result };
    cancelled.value = Boolean(result.cancelled);
    completed.value = true;
    emit('complete', result);
  } catch (error) {
    importError.value = localizeImportError(error);
  } finally {
    running.value = false;
    unlistenProgress?.();
    unlistenProgress = undefined;
    unlistenFinished?.();
    unlistenFinished = undefined;
  }
}

async function cancel() {
  if (cancelling.value) return;
  cancelling.value = true;
  try { await cancelImportAndOrganize(); } catch (error) { importError.value = localizeImportError(error); } finally { cancelling.value = false; }
}

async function continueImport() {
  if (running.value) return;
  completed.value = false;
  cancelled.value = false;
  await startImport();
}

function localizeImportError(error: unknown): string {
  const raw = String(error);
  if (raw.includes('cannot contain one another')) return t('import_organize.error_nested');
  if (raw.includes('must be inside the album')) return t('import_organize.error_destination_outside');
  if (raw.includes('source')) return t('import_organize.error_source');
  return t('import_organize.error_failed', { message: raw });
}

function close() {
  if (running.value) return;
  emit('cancel');
}

onBeforeUnmount(() => {
  disposed = true;
  sourceRequest++;
  unlistenProgress?.();
  unlistenFinished?.();
  unlistenKeydown?.();
  uiStore.removeInputHandler('ImportOrganizeDialog');
  libConfig.destFolder.albumId = null;
  libConfig.destFolder.folderId = null;
  libConfig.destFolder.folderPath = null;
  libConfig.destFolder.selected = false;
});

onMounted(async () => {
  uiStore.pushInputHandler('ImportOrganizeDialog');
  unlistenKeydown = await listen<{ key: string }>('global-keydown', (event: Event<{ key: string }>) => {
    if (!uiStore.isInputActive('ImportOrganizeDialog') || event.payload.key !== 'Escape') return;
    if (running.value) {
      void cancel();
    } else {
      close();
    }
  });
  if (disposed) {
    unlistenKeydown?.();
    return;
  }
  const tree = await fetchFolder(props.album.path, false, config.settings.folderSort);
  if (!disposed) destinationTree.value = tree;
});
</script>

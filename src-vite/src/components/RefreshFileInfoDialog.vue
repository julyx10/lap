<template>
  <ModalDialog :title="$t(freeSpace ? 'menu.file.index_cloud_files' : 'menu.file.refresh_file_info')" :width="440" @cancel="closeOrCancel">
    <section class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
      <div class="font-bold uppercase text-[10px] tracking-widest text-base-content/30">{{ $t('msgbox.file_refresh.status') }}</div>
      <div class="space-y-2 px-1 text-xs" role="status" aria-live="polite">
        <div class="flex items-center justify-between gap-2 text-base-content/75">
          <span>{{ $t(running ? (cancelled ? 'msgbox.file_refresh.cancelling' : 'msgbox.file_refresh.running') : cancelled ? 'msgbox.file_refresh.cancelled' : 'msgbox.file_refresh.complete') }}</span>
          <span class="tabular-nums">{{ processed.toLocaleString() }} / {{ selectionIds.length.toLocaleString() }}</span>
        </div>
        <progress class="progress progress-primary w-full" :value="processed" :max="Math.max(selectionIds.length, 1)"></progress>
        <div class="flex flex-wrap gap-x-3 gap-y-1 text-base-content/60">
          <span>{{ $t('msgbox.file_refresh.succeeded', { count: succeeded.toLocaleString() }) }}</span>
          <span v-if="warned" class="text-warning">{{ $t('msgbox.file_refresh.warnings', { count: warned.toLocaleString() }) }}</span>
          <span v-if="skipped">{{ $t('msgbox.file_refresh.skipped', { count: skipped.toLocaleString() }) }}</span>
          <span :class="{ 'text-error/70': failed > 0 }">{{ $t('msgbox.file_refresh.failed', { count: failed.toLocaleString() }) }}</span>
        </div>
        <div v-if="freeSpace" class="flex gap-3 text-base-content/60">
          <span>{{ $t('msgbox.file_refresh.release_requested', { count: releaseRequested.toLocaleString() }) }}</span>
          <span :title="$t('msgbox.file_refresh.kept_local_hint')">{{ $t('msgbox.file_refresh.kept_local', { count: keptLocal.toLocaleString() }) }}</span>
        </div>
      </div>
      <p v-if="lastWarning" class="text-xs text-warning wrap-break-word" role="status">{{ lastWarning }}</p>
      <p v-if="lastError" class="text-xs text-error/70 wrap-break-word" role="alert">{{ lastError }}</p>
    </section>
    <div class="mt-4 flex justify-end gap-4">
      <button class="t-button-default" :disabled="running && cancelled" @click="closeOrCancel">
        {{ $t(running ? 'msgbox.cancel' : 'msgbox.close') }}
      </button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { config, libConfig } from '@/common/config';
import { getRawDisplayOptions } from '@/common/rawDisplay';
import { useUIStore } from '@/stores/uiStore';
import ModalDialog from '@/components/ModalDialog.vue';

type FileRefreshResult = { file: any; size_delta: number; skipped: boolean; release_requested: boolean; warning?: string | null; error?: string | null };
const props = defineProps<{ fileIds: number[]; libraryId: string; freeSpace?: boolean; finishRefresh: (results: FileRefreshResult[]) => Promise<void> }>();
const emit = defineEmits(['close']);
const uiStore = useUIStore();
const selectionIds = [...new Set(props.fileIds)];
const running = ref(true);
const cancelled = ref(false);
const processed = ref(0);
const succeeded = ref(0);
const skipped = ref(0);
const failed = ref(0);
const warned = ref(0);
const releaseRequested = ref(0);
const lastError = ref('');
const lastWarning = ref('');
const keptLocal = ref(0);
let disposed = false;

function closeOrCancel() {
  if (running.value) {
    cancelled.value = true;
  } else emit('close');
}
function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('RefreshFileInfoDialog')) return;
  if (event.key === 'Escape' || (event.key === 'Enter' && !running.value)) {
    event.preventDefault();
    closeOrCancel();
  }
}
watch(() => libConfig._libraryId, () => { cancelled.value = true; if (!running.value) emit('close'); });
onMounted(async () => {
  uiStore.pushInputHandler('RefreshFileInfoDialog');
  window.addEventListener('keydown', handleKeyDown);
  await runRefresh();
});
async function runRefresh() {
  if (disposed || libConfig._libraryId !== props.libraryId) { running.value = false; return; }
  const results: FileRefreshResult[] = [];
  running.value = true;
  processed.value = 0;
  lastError.value = '';
  lastWarning.value = '';
  // Sequential background commands bound memory and permit cancellation between
  // files. The backend also checks the library under its library-switch lock.
  let batchId: number | undefined;
  try {
    batchId = await invoke<number>('begin_file_refresh_batch', { libraryId: props.libraryId });
    for (const fileId of selectionIds) {
      if (disposed || cancelled.value || libConfig._libraryId !== props.libraryId) {
        cancelled.value = true;
        break;
      }
      try {
        const result = props.freeSpace
          ? await invoke<FileRefreshResult | null>('index_cloud_file', {
              libraryId: props.libraryId, fileId, batchId,
              thumbnailSize: config.settings.thumbnailSize, rawDisplayOptions: getRawDisplayOptions(),
            })
          : await invoke<FileRefreshResult | null>('refresh_selected_file_info', { libraryId: props.libraryId, fileId, batchId, thumbnailSize: config.settings.thumbnailSize, rawDisplayOptions: getRawDisplayOptions() });
        if (result && props.freeSpace) result.release_requested ? releaseRequested.value++ : keptLocal.value++;
        if (result) {
          results.push(result);
          if (result.warning) lastWarning.value = result.warning;
          if (result.error) {
            failed.value++;
            lastError.value = result.error;
          } else if (result.warning) warned.value++;
          else if (result.skipped) skipped.value++;
          else succeeded.value++;
        }
        else failed.value++;
      } catch (error) {
        console.error('Failed to refresh selected file:', fileId, error);
        lastError.value = String(error);
        failed.value++;
      }
      processed.value++;
    }
  } catch (error) {
    lastError.value = String(error);
  } finally {
    if (batchId !== undefined) {
      try { await invoke('end_file_refresh_batch', { batchId }); }
      catch (error) { console.error('Failed to release refresh index:', error); }
    }
    try {
      if (!disposed && libConfig._libraryId === props.libraryId) await props.finishRefresh(results);
    } catch (error) {
      lastError.value = String(error);
    } finally {
      running.value = false;
      if (!disposed && cancelled.value) emit('close');
    }
  }
}
onBeforeUnmount(() => {
  disposed = true;
  cancelled.value = true;
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('RefreshFileInfoDialog');
});
</script>

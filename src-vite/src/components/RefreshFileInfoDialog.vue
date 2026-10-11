<template>
  <ModalDialog :title="$t('menu.file.refresh_file_info')" :width="440" @cancel="closeOrCancel">
    <section class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
      <div class="font-bold uppercase text-[10px] tracking-widest text-base-content/30">{{ $t('msgbox.file_refresh.status') }}</div>
      <div class="space-y-2 px-1 text-xs" role="status" aria-live="polite">
        <div class="flex items-center justify-between gap-2 text-base-content/75">
          <span>{{ $t(running ? (cancelled ? 'msgbox.file_refresh.cancelling' : 'msgbox.file_refresh.running') : cancelled ? 'msgbox.file_refresh.cancelled' : 'msgbox.file_refresh.complete') }}</span>
          <span class="tabular-nums">{{ processed.toLocaleString() }} / {{ fileIds.length.toLocaleString() }}</span>
        </div>
        <progress class="progress progress-primary w-full" :value="processed" :max="Math.max(fileIds.length, 1)"></progress>
        <div class="flex gap-3 text-base-content/60">
          <span>{{ $t('msgbox.file_refresh.succeeded', { count: succeeded.toLocaleString() }) }}</span>
          <span :class="{ 'text-error/70': failed > 0 }">{{ $t('msgbox.file_refresh.failed', { count: failed.toLocaleString() }) }}</span>
        </div>
      </div>
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
import { libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import ModalDialog from '@/components/ModalDialog.vue';

type FileRefreshResult = { file: any; size_delta: number; skipped: boolean };
const props = defineProps<{ fileIds: number[]; libraryId: string; finishRefresh: (results: FileRefreshResult[]) => Promise<void> }>();
const emit = defineEmits(['close']);
const uiStore = useUIStore();
const running = ref(true);
const cancelled = ref(false);
const processed = ref(0);
const succeeded = ref(0);
const failed = ref(0);
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
watch(() => libConfig._libraryId, () => { cancelled.value = true; });
onMounted(async () => {
  const results: FileRefreshResult[] = [];
  uiStore.pushInputHandler('RefreshFileInfoDialog');
  window.addEventListener('keydown', handleKeyDown);
  // Sequential background commands bound memory and permit cancellation between
  // files. The backend also checks the library under its library-switch lock.
  try {
    for (const fileId of props.fileIds) {
      if (disposed || cancelled.value || libConfig._libraryId !== props.libraryId) {
        cancelled.value = true;
        break;
      }
      try {
        const result = await invoke<FileRefreshResult | null>('refresh_selected_file_info', { libraryId: props.libraryId, fileId });
        if (result) {
          results.push(result);
          succeeded.value++;
        }
        else failed.value++;
      } catch (error) {
        console.error('Failed to refresh selected file:', fileId, error);
        failed.value++;
      }
      processed.value++;
    }
  } finally {
    try {
      if (!disposed && libConfig._libraryId === props.libraryId) await props.finishRefresh(results);
    } finally {
      running.value = false;
      if (!disposed && cancelled.value) emit('close');
    }
  }
});
onBeforeUnmount(() => {
  disposed = true;
  cancelled.value = true;
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('RefreshFileInfoDialog');
});
</script>

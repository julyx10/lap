<template>
  <ModalDialog :title="$t('msgbox.batch_rename.title')" :width="640" position-key="batch-rename" @cancel="close">
    <form class="space-y-3" @submit.prevent="submit">
      <section class="rounded-box border border-base-content/10 bg-base-300/20 p-3 shadow-sm">
        <div class="grid grid-cols-[minmax(0,1fr)_7rem_7rem] gap-2">
          <label class="min-w-0 space-y-1">
            <span class="block text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('msgbox.batch_rename.name') }}</span>
            <input ref="nameInput" v-model="name" :disabled="saving" :placeholder="$t('msgbox.batch_rename.placeholder')" class="input input-sm text-xs w-full" />
          </label>
          <label class="min-w-0 space-y-1">
            <span class="block text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('msgbox.batch_rename.start') }}</span>
            <input v-model.number="start" :disabled="saving" type="number" min="1" max="4294967295" step="1" class="input input-sm text-xs w-full" />
          </label>
          <label class="min-w-0 space-y-1">
            <span class="block text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('msgbox.batch_rename.digits') }}</span>
            <select v-model.number="digits" :disabled="saving" class="select select-sm text-xs w-full">
              <option :value="0">{{ $t('msgbox.batch_rename.no_padding') }}</option>
              <option v-for="n in 6" :key="n" :value="n">{{ n }}</option>
            </select>
          </label>
        </div>
      </section>
      <section class="rounded-box border border-base-content/10 bg-base-300/20 p-3 shadow-sm space-y-2">
        <div class="flex items-center justify-between min-h-4">
          <span class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('msgbox.batch_rename.preview') }}</span>
          <span v-if="preview && !pending" class="text-xs text-base-content/40">{{ $t('msgbox.batch_rename.count', { count: preview.items.length }) }}</span>
          <span v-if="pending" class="loading loading-spinner loading-xs"></span>
        </div>
        <div class="max-h-64 overflow-auto rounded-box border border-base-content/10">
          <table class="table table-xs text-xs text-base-content/70 w-full table-fixed">
            <thead class="sticky top-0 z-10 bg-base-200 text-[10px] uppercase tracking-widest font-bold text-base-content/30"><tr><th>{{ $t('msgbox.batch_rename.before') }}</th><th>{{ $t('msgbox.batch_rename.after') }}</th></tr></thead>
            <tbody>
              <tr v-for="item in visibleItems" :key="item.id">
                <td class="truncate" :title="item.oldName">{{ item.oldName }}</td>
                <td class="truncate" :title="item.newName">{{ item.newName }}<span v-if="item.companions" class="text-base-content/40"> (+{{ item.companions }})</span></td>
              </tr>
            </tbody>
          </table>
        </div>
        <p v-if="preview && preview.items.length > 100" class="text-xs text-base-content/40">{{ $t('msgbox.batch_rename.preview_limit') }}</p>
        <p v-if="error" class="text-xs text-error whitespace-pre-wrap max-h-24 overflow-auto" role="alert">{{ error }}</p>
      </section>
      <div class="flex justify-end gap-2 pt-1">
        <button type="button" class="t-button-default" :disabled="saving" @click="close">{{ $t('msgbox.cancel') }}</button>
        <button type="submit" class="t-button-primary" :disabled="!canSubmit">
          <span v-if="saving" class="loading loading-spinner loading-xs"></span>
          {{ $t('menu.file.rename') }}
        </button>
      </div>
    </form>
  </ModalDialog>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import ModalDialog from '@/components/ModalDialog.vue';
import { batchRenameFiles, previewBatchRename } from '@/common/api';
import { config } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';

const props = defineProps<{ files: any[]; libraryId: string }>();
const emit = defineEmits(['cancel', 'done']);
const { t } = useI18n();
const uiStore = useUIStore();
const nameInput = ref<HTMLInputElement | null>(null);
const name = ref(String(props.files[0]?.name || t('msgbox.batch_rename.placeholder')).replace(/\.[^.]+$/, ''));
const start = ref(1);
const digits = ref(0);
const pending = ref(false);
const saving = ref(false);
const preview = ref<any>(null);
const error = ref('');
let timer: ReturnType<typeof setTimeout> | undefined;
let generation = 0;
let disposed = false;
const options = computed(() => ({ name: name.value, start: start.value, digits: digits.value }));
const ids = props.files.map(file => Number(file.id));
const groupRawJpegPairs = Boolean(config.settings.groupRawJpegPairs);
const visibleItems = computed(() => (preview.value?.items || []).slice(0, 100));
const canSubmit = computed(() => !pending.value && !saving.value && !error.value && preview.value?.items.length > 0);

function refreshPreview(delay = 0) {
  const request = ++generation;
  clearTimeout(timer);
  error.value = '';
  pending.value = true;
  const load = async () => {
    if (!name.value.trim() || !Number.isInteger(start.value) || start.value < 1 || start.value > 4294967295) {
      if (request === generation) { error.value = t('msgbox.batch_rename.invalid'); pending.value = false; }
      return;
    }
    try {
      const result = await previewBatchRename(props.libraryId, ids, options.value, groupRawJpegPairs);
      if (disposed || request !== generation) return;
      if (preview.value && preview.value.items.length === result.items.length
        && preview.value.items.every((item: any, index: number) => item.id === result.items[index].id && item.oldName === result.items[index].oldName)) {
        for (let index = 0; index < result.items.length; index++) {
          preview.value.items[index].newName = result.items[index].newName;
          preview.value.items[index].companions = result.items[index].companions;
        }
        preview.value.errors = result.errors;
      } else {
        preview.value = result;
      }
      error.value = result.errors.join('\n');
    } catch (reason) {
      if (!disposed && request === generation) error.value = String(reason);
    } finally {
      if (!disposed && request === generation) pending.value = false;
    }
  };
  if (delay) timer = setTimeout(load, delay);
  else void load();
}
watch(options, (_, previous) => refreshPreview(previous ? 200 : 0), { immediate: true, flush: 'sync' });

async function submit() {
  if (!canSubmit.value) return;
  saving.value = true;
  try {
    const result = await batchRenameFiles(props.libraryId, ids, options.value, preview.value.items, groupRawJpegPairs);
    emit('done', result);
  } catch (reason) {
    error.value = String(reason);
  } finally {
    saving.value = false;
  }
}
function close() { if (!saving.value) emit('cancel'); }
function keydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && uiStore.isInputActive('BatchRenameDialog')) { event.preventDefault(); event.stopPropagation(); close(); }
}
onMounted(async () => {
  uiStore.pushInputHandler('BatchRenameDialog');
  window.addEventListener('keydown', keydown);
  await nextTick();
  nameInput.value?.select();
});
onBeforeUnmount(() => {
  disposed = true;
  generation++;
  clearTimeout(timer);
  window.removeEventListener('keydown', keydown);
  uiStore.removeInputHandler('BatchRenameDialog');
});
</script>

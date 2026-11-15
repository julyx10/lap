<template>
  <ModalDialog :title="$t('file_info.edit_metadata')" :width="460" position-key="edit-metadata" @cancel="close">
    <form class="space-y-3 overflow-auto max-h-[70vh]" @submit.prevent="save" @keydown.esc.stop.prevent="close">
      <section aria-labelledby="edit-metadata-details" class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
        <h2 id="edit-metadata-details" class="font-bold uppercase text-[10px] tracking-widest text-base-content/30">
          {{ $t('file_info.edit_metadata_details') }}
        </h2>
        <p v-if="!visibleFields.length" class="px-1 text-sm text-base-content/70">
          {{ $t('file_info.edit_metadata_no_fields') }}
        </p>
        <div v-else class="w-full grid grid-cols-[84px_minmax(0,1fr)] gap-x-4 gap-y-1.5 px-1 text-xs">
          <template v-for="field in visibleFields" :key="field.key">
            <label :for="'metadata-' + field.key" class="min-h-6 flex items-start pt-1 text-[11px] text-base-content/30">
              {{ $t(field.label) }}
            </label>
            <component :is="['comments', 'e_artist'].includes(field.key) ? 'textarea' : 'input'"
              :id="'metadata-' + field.key"
              :ref="el => { if (el) inputs[field.key] = el as HTMLElement; }"
              :placeholder="$t('file_info.metadata_placeholders.' + field.key)"
              :value="values[field.key]" :disabled="saving"
              :class="['comments', 'e_artist'].includes(field.key)
                ? 'w-full textarea textarea-sm min-h-14 max-h-50 px-1.5 text-[12px] font-medium'
                : 'w-full input input-xs h-6 px-1.5 text-[12px] font-medium'"
              @input="update(field.key, $event)" />
          </template>
        </div>
      </section>
      <p v-if="failed" role="alert" class="text-error text-sm">{{ $t('file_info.metadata_save_failed') }}</p>
      <div class="flex justify-end gap-2">
        <button type="button" class="t-button-default" :disabled="saving" @click="close">{{ $t('msgbox.cancel') }}</button>
        <button v-if="visibleFields.length" type="submit" class="t-button-primary" :disabled="saving">{{ $t('msgbox.ok') }}</button>
      </div>
    </form>
  </ModalDialog>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onBeforeUnmount, reactive, ref } from 'vue';
import ModalDialog from './ModalDialog.vue';
import { config, libConfig } from '@/common/config';
import { editFileMetadata } from '@/common/api';
import { editableMetadataFields, normalizeFileInfoFields } from '@/common/fileInfoFields';
import { useUIStore } from '@/stores/uiStore';

const props = defineProps<{ files: Record<string, any>[]; focusField?: string }>();
const emit = defineEmits(['saved', 'cancel']);
const libraryId = libConfig._libraryId;
const files = [...props.files];
const fileIds = files.map(file => Number(file.id));
const values = reactive(Object.fromEntries(editableMetadataFields.map(({ key }) => {
  const first = files[0]?.[key] ?? '';
  return [key, files.every(file => (file[key] ?? '') === first) ? first : ''];
})));
const visibleFields = computed(() => {
  const selected = new Set(normalizeFileInfoFields(config.infoPanel.fields));
  return editableMetadataFields.filter(field => selected.has(`organization.${field.key}`));
});
const touched = new Set<string>();
const inputs: Record<string, HTMLElement> = {};
const saving = ref(false);
const failed = ref(false);
const ui = useUIStore();
function update(key: string, event: Event) {
  values[key] = (event.target as HTMLInputElement).value;
  touched.add(key);
}
function close() {
  if (!saving.value) emit('cancel');
}
async function save() {
  if (saving.value) return;
  const changes = Object.fromEntries(visibleFields.value.filter(field => touched.has(field.key)).map(({ key }) => [key, values[key].trim()])
    .filter(([key, value]) => files.some(file => file[key] !== value)));
  saving.value = true;
  failed.value = false;
  try {
    if (Object.keys(changes).length) await editFileMetadata(libraryId, fileIds, changes);
    if (libraryId !== libConfig._libraryId) {
      emit('cancel');
      return;
    }
    emit('saved', files, changes);
  } catch {
    failed.value = true;
  } finally {
    saving.value = false;
  }
}
onMounted(async () => {
  ui.pushInputHandler('EditMetadataDialog');
  await nextTick();
  const field = visibleFields.value.find(field => field.key === props.focusField) ?? visibleFields.value[0];
  if (field) inputs[field.key]?.focus();
});
onBeforeUnmount(() => ui.removeInputHandler('EditMetadataDialog'));
</script>

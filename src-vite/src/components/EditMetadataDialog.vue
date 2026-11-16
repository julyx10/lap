<template>
  <ModalDialog :title="$t('file_info.edit_metadata')" :width="540" position-key="edit-metadata" @cancel="close">
    <form class="space-y-3 overflow-auto max-h-[70vh]" @submit.prevent="save" @keydown.esc.stop.prevent="close">
      <div class="rounded-box p-2 bg-base-300/30 border border-base-content/5 shadow-sm">
        <p v-if="!visibleFields.length" class="px-1 text-sm text-base-content/70">
          {{ $t('file_info.edit_metadata_no_fields') }}
        </p>
        <div v-else class="grid grid-cols-2 gap-x-3 gap-y-3 px-1">
          <label
            v-for="field in visibleFields"
            :key="field.key"
            :for="'metadata-' + field.key"
            class="min-w-0 flex flex-col gap-1"
            :class="{ 'col-span-2': ['comments', 'e_artist', 'e_copyright'].includes(field.key) }"
          >
            <span class="flex items-center justify-between gap-2 text-[11px] text-base-content/30">
              <span>{{ $t(field.label) }}</span>
              <span v-if="field.key === 'e_artist'" class="shrink-0 text-[10px]">
                {{ $t('file_info.metadata_placeholders.e_artist_hint') }}
              </span>
            </span>
            <component
              :is="['comments', 'e_artist'].includes(field.key) ? 'textarea' : 'input'"
              :id="'metadata-' + field.key"
              :ref="el => { if (el) inputs[field.key] = el as HTMLElement; }"
              :placeholder="metadataPlaceholder(field.key)"
              :value="values[field.key]"
              :disabled="saving"
              :class="field.key === 'comments'
                ? 'w-full textarea textarea-sm min-h-20 max-h-50 px-1.5 text-[12px] font-medium'
                : field.key === 'e_artist'
                  ? 'w-full textarea textarea-sm min-h-14 max-h-32 px-1.5 text-[12px] font-medium'
                  : 'w-full input input-xs h-8 px-1.5 text-[12px] font-medium'"
              @input="update(field.key, $event)"
            />
          </label>
        </div>
      </div>
      <p v-if="failed" role="alert" class="text-error text-sm">{{ $t('file_info.metadata_save_failed') }}</p>
      <div class="flex justify-end gap-2">
        <button type="button" class="t-button-default" :disabled="saving" @click="close">{{ $t('msgbox.cancel') }}</button>
        <button v-if="visibleFields.length" type="submit" class="t-button-primary" :disabled="saving">{{ $t('msgbox.save') }}</button>
      </div>
    </form>
  </ModalDialog>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onBeforeUnmount, reactive, ref } from 'vue';
import ModalDialog from './ModalDialog.vue';
import { config, libConfig } from '@/common/config';
import { editFileMetadata } from '@/common/api';
import { editableMetadataFields, getVisibleEditableMetadataFields } from '@/common/fileInfoFields';
import { useUIStore } from '@/stores/uiStore';
import { useI18n } from 'vue-i18n';

const props = defineProps<{ files: Record<string, any>[]; focusField?: string }>();
const emit = defineEmits(['saved', 'cancel']);
const libraryId = libConfig._libraryId;
const files = [...props.files];
const fileIds = files.map(file => Number(file.id));
const values = reactive(Object.fromEntries(editableMetadataFields.map(({ key }) => {
  const first = files[0]?.[key] ?? '';
  return [key, files.every(file => (file[key] ?? '') === first) ? first : ''];
})));
const visibleFields = computed(() => getVisibleEditableMetadataFields(config.infoPanel.fields));
const touched = new Set<string>();
const inputs: Record<string, HTMLElement> = {};
const saving = ref(false);
const failed = ref(false);
const ui = useUIStore();
const { t } = useI18n();
const currentYear = new Date().getFullYear();
const metadataPlaceholder = (key: string) => key === 'e_copyright'
  ? t('file_info.metadata_placeholders.e_copyright', { year: currentYear })
  : t(`file_info.metadata_placeholders.${key}`);
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
